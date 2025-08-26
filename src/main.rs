// main.rs
#![allow(unused_imports)]
#![allow(dead_code)]

mod line;
mod framebuffer;
mod maze;
mod caster;
mod player;
mod minimap;
mod renderer;
mod sprite;
mod textures;

use line::line;
use maze::{Maze,load_maze};
use caster::{cast_ray, Intersect};
use framebuffer::Framebuffer;
use player::{Player, process_events};

use raylib::prelude::*;
use std::thread;
use std::time::Duration;
use std::env;
use std::f32::consts::PI;

 

fn main() {
    // Allow overriding resolution via command-line: cargo run -- <width> <height>
    let args: Vec<String> = env::args().collect();
    let mut window_width: i32 = 1300;
    let mut window_height: i32 = 900;
    if args.len() >= 3 {
        match (args[1].parse::<i32>(), args[2].parse::<i32>()) {
            (Ok(w), Ok(h)) => {
                if w > 200 && h > 200 {
                    window_width = w;
                    window_height = h;
                } else {
                    eprintln!("[warn] provided resolution too small, using default {}x{}", window_width, window_height);
                }
            }
            _ => {
                eprintln!("[warn] invalid resolution arguments, expected two integers, using default {}x{}", window_width, window_height);
            }
        }
    } else {
        eprintln!("[info] run with \"<program> <width> <height>\" to override resolution. Using default {}x{}", window_width, window_height);
    }
    let block_size = 100;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Raycaster Example")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    // render_scale reduces the internal framebuffer resolution to improve FPS.
    // e.g. render_scale = 2 renders to (width/2 x height/2) and scales up when drawing.
    let render_scale: u32 = 2; // increase to 3/4 for better perf, set to 1 for native resolution
    let fb_w = (window_width as u32).saturating_div(render_scale);
    let fb_h = (window_height as u32).saturating_div(render_scale);
    let mut framebuffer = Framebuffer::new(fb_w, fb_h);
    framebuffer.set_background_color(Color::new(50, 50, 100, 255));

    // load textures atlas (optional - will fallback to procedural patterns)
    let textures = textures::TextureAtlas::new();

        let maze = load_maze("maze.txt");

        // DEBUG: print working directory and the resolved path of maze.txt so we know which file is loaded
        if let Ok(cwd) = env::current_dir() {
            eprintln!("[debug] CWD: {}", cwd.display());
        }
        match std::fs::canonicalize("maze.txt") {
            Ok(p) => eprintln!("[debug] maze.txt -> {}", p.display()),
            Err(e) => eprintln!("[debug] couldn't canonicalize maze.txt: {}", e),
        }
        eprintln!("[debug] loaded maze rows = {}", maze.len());
    let mut player = Player {
        pos: Vector2::new(150.0, 150.0),
        a: PI / 3.0,
        fov: PI / 3.0,
    };

    // start with mouse capture disabled; user can toggle with ESC
    let mut capture_mouse = false;

    // load NPCs from maze
    let mut npcs = sprite::load_npcs_from_maze(&maze, block_size);

    while !window.window_should_close() {
        // 1. clear framebuffer
        framebuffer.clear();

    // 2. move the player on user input (with collision checks)
    process_events(&mut player, &window, &maze, block_size);

        // update NPCs
        sprite::update_npcs(&mut npcs, &player, &maze, block_size);

    // 3. draw stuff: always render 3D world and a stylized minimap
    // pass column_step derived from render_scale to the renderer (more aggressive when downscaling)
    let column_step = render_scale as usize; 
    renderer::render_world(&mut framebuffer, &maze, block_size, &player, &textures, &npcs, column_step);
    let minimap_scale = 14; // increased pixels per cell for bigger minimap
    // place minimap at 12,12 offset
    minimap::render_minimap(&mut framebuffer, &maze, minimap_scale, &player, 12, 12, block_size, &npcs);

    // 4. swap buffers (draw framebuffer and overlay FPS)
    let fps = window.get_fps();
    framebuffer.swap_buffers(&mut window, &raylib_thread, Some(fps as i32));
        // toggle mouse capture with ESC key (currently only toggles state; we avoid forcing
        // SetMousePosition each frame since that can zero mouse delta on some platforms)
        if window.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            capture_mouse = !capture_mouse;
            if capture_mouse {
                // hide cursor when capture is enabled
                window.hide_cursor();
            } else {
                window.show_cursor();
            }
        }

        

        thread::sleep(Duration::from_millis(16));
    }
}
