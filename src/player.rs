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
    // Movement: WASD -> forward/back + strafing. Mouse -> camera yaw.
    const MOVE_SPEED: f32 = 10.0;
    const MOUSE_SENSITIVITY: f32 = 0.0035;

    // Mouse look: apply relative mouse delta only while right mouse button is held
    if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
        let md = rl.get_mouse_delta();
        player.a -= md.x as f32 * MOUSE_SENSITIVITY;
    }

    // WASD: W forward, S backward, A left strafe, D right strafe
    let mut forward: f32 = 0.0;
    let mut strafe: f32 = 0.0;
    if rl.is_key_down(KeyboardKey::KEY_W) { forward += 1.0; }
    if rl.is_key_down(KeyboardKey::KEY_S) { forward -= 1.0; }
    if rl.is_key_down(KeyboardKey::KEY_D) { strafe += 1.0; }
    if rl.is_key_down(KeyboardKey::KEY_A) { strafe -= 1.0; }

    if forward != 0.0 || strafe != 0.0 {
        // movement vector in world coordinates
        let fx = player.a.cos();
        let fy = player.a.sin();
        let sx = (player.a + PI / 2.0).cos();
        let sy = (player.a + PI / 2.0).sin();

        let dx = (forward * fx + strafe * sx) * MOVE_SPEED;
        let dy = (forward * fy + strafe * sy) * MOVE_SPEED;

        let new_x = player.pos.x + dx;
        let new_y = player.pos.y + dy;

        // collision with sliding: try full move, then X-only and Y-only
        if can_move_to(maze, new_x, new_y, block_size) {
            player.pos.x = new_x;
            player.pos.y = new_y;
        } else {
            if can_move_to(maze, new_x, player.pos.y, block_size) {
                player.pos.x = new_x;
            }
            if can_move_to(maze, player.pos.x, new_y, block_size) {
                player.pos.y = new_y;
            }
        }
    }
}
