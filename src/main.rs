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
    let window_width = 1300;
    let window_height = 900;
    let block_size = 100;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Raycaster Example")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32);
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
    renderer::render_world(&mut framebuffer, &maze, block_size, &player, &textures, &npcs);
    let minimap_scale = 14; // increased pixels per cell for bigger minimap
    // place minimap at 12,12 offset
    minimap::render_minimap(&mut framebuffer, &maze, minimap_scale, &player, 12, 12, block_size, &npcs);

    // 4. swap buffers (draw framebuffer and overlay FPS)
    let fps = window.get_fps();
    framebuffer.swap_buffers(&mut window, &raylib_thread, Some(fps as i32));

        thread::sleep(Duration::from_millis(16));
    }
}
