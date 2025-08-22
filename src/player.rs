// player.rs

use raylib::prelude::*;
use std::f32::consts::PI;
use crate::maze::Maze;

pub struct Player {
    pub pos: Vector2,
    pub a: f32,
    pub fov: f32, // field of view
}

// Check whether a point (x,y) in world coordinates is inside a free cell of the maze
pub fn can_move_to(maze: &Maze, x: f32, y: f32, block_size: usize) -> bool {
    if maze.is_empty() {
        return true;
    }
    if x < 0.0 || y < 0.0 {
        return false;
    }
    let i = (x as usize) / block_size;
    let j = (y as usize) / block_size;
    if j >= maze.len() {
        return false;
    }
    if maze[j].is_empty() || i >= maze[j].len() {
        return false;
    }
    // treat 'R' (sprite NPC) as non-blocking so player can walk around it
    maze[j][i] == ' ' || maze[j][i] == 'R'
}

// Process input and perform movement with simple collision against maze walls.
// Uses axis-aligned sliding: if full move collides, tries X-only and Y-only moves.
pub fn process_events(player: &mut Player, rl: &RaylibHandle, maze: &Maze, block_size: usize) {
    const MOVE_SPEED: f32 = 10.0;
    // Slightly faster rotation (~2 degrees per frame)
    const ROTATION_SPEED: f32 = PI / 90.0;
    const MOUSE_SENSITIVITY: f32 = 0.004;

    // Use WASD: W forward, S backward, A rotate left, D rotate right
    if rl.is_key_down(KeyboardKey::KEY_A) {
        player.a += ROTATION_SPEED;
    }
    if rl.is_key_down(KeyboardKey::KEY_D) {
        player.a -= ROTATION_SPEED;
    }

    // Right-click + mouse movement rotates the player horizontally
    let md = rl.get_mouse_delta();
    if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
        // move mouse right -> rotate right (decrease angle)
        player.a -= md.x as f32 * MOUSE_SENSITIVITY;
    }

    let mut dx = 0.0_f32;
    let mut dy = 0.0_f32;

    if rl.is_key_down(KeyboardKey::KEY_S) {
        dx -= MOVE_SPEED * player.a.cos();
        dy -= MOVE_SPEED * player.a.sin();
    }
    if rl.is_key_down(KeyboardKey::KEY_W) {
        dx += MOVE_SPEED * player.a.cos();
        dy += MOVE_SPEED * player.a.sin();
    }

    // Apply movement only if requested (rotation via mouse still works)
    if !(dx == 0.0 && dy == 0.0) {
        let new_x = player.pos.x + dx;
        let new_y = player.pos.y + dy;

        // If full move is allowed, do it
        if can_move_to(maze, new_x, new_y, block_size) {
            player.pos.x = new_x;
            player.pos.y = new_y;
        } else {
            // Otherwise try sliding on X then Y
            if can_move_to(maze, new_x, player.pos.y, block_size) {
                player.pos.x = new_x;
            }
            if can_move_to(maze, player.pos.x, new_y, block_size) {
                player.pos.y = new_y;
            }
        }
    }
}
