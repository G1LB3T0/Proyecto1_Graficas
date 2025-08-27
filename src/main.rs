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
mod menu;
mod audio;
mod anim;

use line::line;
use maze::{Maze,load_maze};
use caster::{cast_ray, Intersect};
use framebuffer::Framebuffer;
use player::{Player, process_events};

use raylib::prelude::*;
use std::ffi::CString;
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


    // audio manager: encapsulates audio init/play/stop/update
    let mut audio = audio::AudioManager::new();
    audio.init();
    audio.play_menu_track();

    // show main menu and handle selection
    match menu::run_menu(&mut window, &raylib_thread, &mut framebuffer, &textures, &mut audio) {
        menu::MenuAction::Start => {
            // stop menu music and start gameplay music
            audio.stop_unload();
            audio.play_game_track();
        }
        menu::MenuAction::Quit => {
            audio.cleanup();
            return;
        }
    }

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

    // start with mouse capture enabled for better FPS-style controls
    let mut capture_mouse = true;
    window.hide_cursor(); // hide cursor initially

    // load NPCs from maze
    let mut npcs = sprite::load_npcs_from_maze(&maze, block_size);
    // load coins from maze
    let mut coins = sprite::load_coins_from_maze(&maze, block_size);
    let mut total_coins_collected = 0;
    // fog-of-war discovered grid for the minimap (initialized to false)
    let mut discovered: Vec<Vec<bool>> = maze.iter().map(|r| vec![false; r.len()]).collect();

    while !window.window_should_close() {
        // 1. clear framebuffer
        framebuffer.clear();

    // 2. move the player on user input (with collision checks)
    // doors open when all coins are collected
    let doors_open = total_coins_collected >= coins.len();
    process_events(&mut player, &mut window, &maze, block_size, capture_mouse, doors_open);

    // check if player has escaped (is standing on the door position when doors are open)
    let player_escaped = doors_open && {
        let player_grid_x = (player.pos.x / block_size as f32) as usize;
        let player_grid_y = (player.pos.y / block_size as f32) as usize;
        // Check if player is on a door position ('G' in the maze)
        if player_grid_y < maze.len() && player_grid_x < maze[player_grid_y].len() {
            maze[player_grid_y][player_grid_x] == 'G'
        } else {
            false
        }
    };

        // update NPCs and check for collision (player death)
        let doors_open = total_coins_collected >= coins.len();
        let player_dead = sprite::update_npcs(&mut npcs, &player, &maze, block_size, doors_open);
        
        // update coins and check for collection
        let coins_collected_this_frame = sprite::update_coins(&mut coins, &player, block_size);
        total_coins_collected += coins_collected_this_frame;

        // check for victory condition (player escaped through the door)
        if player_escaped {
            // Victory screen: Enter to restart, Q to quit
            loop {
                framebuffer.clear();
                
                // poll keys before drawing to avoid borrow conflicts
                if window.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    // reset player, npcs, coins, discovered and break to resume game
                    player.pos = Vector2::new(150.0, 150.0);
                    player.a = PI / 3.0;
                    npcs = sprite::load_npcs_from_maze(&maze, block_size);
                    coins = sprite::load_coins_from_maze(&maze, block_size);
                    total_coins_collected = 0;
                    discovered = maze.iter().map(|r| vec![false; r.len()]).collect();
                    break;
                }
                if window.is_key_pressed(KeyboardKey::KEY_Q) {
                    // cleanup audio and quit
                    audio.cleanup();
                    return;
                }

                // draw with raylib (query sizes first)
                let screen_w = window.get_screen_width();
                let screen_h = window.get_screen_height();
                
                // Clear framebuffer and draw victory background
                let fbw = framebuffer.width;
                let fbh = framebuffer.height;
                
                // If victoria texture exists, stretch it to cover the entire framebuffer
                for y in 0..fbh {
                    for x in 0..fbw {
                        let u = x as f32 / fbw as f32;
                        let v = y as f32 / fbh as f32;
                        let col = textures.sample_victoria(u, v);
                        framebuffer.set_current_color(col);
                        framebuffer.set_pixel(x, y);
                    }
                }
                
                if let Ok(texture) = window.load_texture_from_image(&raylib_thread, &framebuffer.color_buffer) {
                    let mut d = window.begin_drawing(&raylib_thread);
                    let src = Rectangle::new(0.0,0.0,framebuffer.width as f32, framebuffer.height as f32);
                    let dest = Rectangle::new(0.0,0.0,screen_w as f32, screen_h as f32);
                    d.draw_texture_pro(&texture, src, dest, Vector2::new(0.0,0.0), 0.0, Color::WHITE);
                    
                    // Draw controls
                    d.draw_text("ENTER = REINICIAR  Q = SALIR", 24, 56, 16, Color::WHITE);
                }
                
                thread::sleep(Duration::from_millis(16));
            }
        }

    if player_dead {
            // simple Game Over screen: Enter to restart, Q to quit
            loop {
                framebuffer.clear();
                // draw current framebuffer scene briefly
                let title = "GAME OVER";

                // poll keys before drawing to avoid borrow conflicts
                if window.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    // reset player, npcs, coins, discovered and break to resume game
                    player.pos = Vector2::new(150.0, 150.0);
                    player.a = PI / 3.0;
                    npcs = sprite::load_npcs_from_maze(&maze, block_size);
                    coins = sprite::load_coins_from_maze(&maze, block_size);
                    total_coins_collected = 0;
                    discovered = maze.iter().map(|r| vec![false; r.len()]).collect();
                    break;
                }
                if window.is_key_pressed(KeyboardKey::KEY_Q) {
                    // cleanup audio and quit
                    audio.cleanup();
                    return;
                }

                // draw with raylib (query sizes first)
                let screen_w = window.get_screen_width();
                let screen_h = window.get_screen_height();
                    // If game over texture exists, stretch it to cover the entire framebuffer
                    if textures.game_over.is_some() {
                        // fill framebuffer by sampling the game_over texture stretched to fb size
                        let fbw = framebuffer.width as u32;
                        let fbh = framebuffer.height as u32;
                        for y in 0..fbh {
                            for x in 0..fbw {
                                let u = x as f32 / fbw as f32;
                                let v = y as f32 / fbh as f32;
                                let col = textures.sample_gameover(u, v);
                                framebuffer.set_current_color(col);
                                framebuffer.set_pixel(x, y);
                            }
                        }
                        // draw framebuffer to screen and overlay controls text
                        if let Ok(texture) = window.load_texture_from_image(&raylib_thread, &framebuffer.color_buffer) {
                            let mut d = window.begin_drawing(&raylib_thread);
                            let src = Rectangle::new(0.0,0.0,framebuffer.width as f32, framebuffer.height as f32);
                            let dest = Rectangle::new(0.0,0.0,screen_w as f32, screen_h as f32);
                            d.draw_texture_pro(&texture, src, dest, Vector2::new(0.0,0.0), 0.0, Color::WHITE);
                            d.draw_text("ENTER = REINICIAR  Q = SALIR", 24, 56, 16, Color::WHITE);
                        }
                    } else if let Ok(texture) = window.load_texture_from_image(&raylib_thread, &framebuffer.color_buffer) {
                        let mut d = window.begin_drawing(&raylib_thread);
                        let src = Rectangle::new(0.0,0.0,framebuffer.width as f32, framebuffer.height as f32);
                        let dest = Rectangle::new(0.0,0.0,screen_w as f32, screen_h as f32);
                        d.draw_texture_pro(&texture, src, dest, Vector2::new(0.0,0.0), 0.0, Color::WHITE);
                        d.draw_rectangle(10, 10, 300, 80, Color::new(0,0,0,160));
                        d.draw_text(title, 24, 20, 40, Color::RAYWHITE);
                        d.draw_text("ENTER = REINICIAR  Q = SALIR", 24, 56, 16, Color::WHITE);
                    }
                thread::sleep(Duration::from_millis(16));
            }
        }

    // 3. draw stuff: always render 3D world and a stylized minimap
    // pass column_step derived from render_scale to the renderer (more aggressive when downscaling)
    let column_step = render_scale as usize; 
    // doors open when all coins are collected
    let doors_open = total_coins_collected >= coins.len();
    renderer::render_world(&mut framebuffer, &maze, block_size, &player, &textures, &npcs, &coins, column_step, doors_open);
    let minimap_scale = 14; // increased pixels per cell for bigger minimap
    // place minimap at 12,12 offset
    minimap::render_minimap(&mut framebuffer, &maze, minimap_scale, &player, 12, 12, block_size, &npcs, &coins, &mut discovered);

    // 4. swap buffers (draw framebuffer with coin counter and FPS)
    let fps = window.get_fps();
    framebuffer.swap_buffers_with_coins(&mut window, &raylib_thread, Some(fps as i32), total_coins_collected, coins.len());
    
    // update music streaming buffers each frame
    audio.update();
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
