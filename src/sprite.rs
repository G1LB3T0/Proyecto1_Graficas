// sprite.rs

use raylib::prelude::*;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;
use crate::textures::TextureAtlas;
use crate::player::can_move_to;
use std::collections::VecDeque;

// Helpers: grid-based Bresenham line check for line-of-sight and a BFS to get the next
// walkable cell towards the goal when walls block the straight line.

fn cell_indices_from_pos(pos_x: f32, pos_y: f32, block_size: usize) -> (isize,isize) {
    let i = (pos_x / block_size as f32).floor() as isize;
    let j = (pos_y / block_size as f32).floor() as isize;
    (i,j)
}

fn in_bounds(maze: &Maze, i: isize, j: isize) -> bool {
    if j < 0 { return false; }
    let rows = maze.len() as isize;
    if j >= rows { return false; }
    let cols = maze[j as usize].len() as isize;
    if i < 0 || i >= cols { return false; }
    true
}

fn is_walkable_cell(maze: &Maze, i: isize, j: isize) -> bool {
    if !in_bounds(maze, i, j) { return false; }
    let c = maze[j as usize][i as usize];
    c == ' ' || c == 'R'
}

// Bresenham integer line between grid cells to test LOS (returns true when no wall cell encountered)
fn line_of_sight(maze: &Maze, from_x: f32, from_y: f32, to_x: f32, to_y: f32, block_size: usize) -> bool {
    let (mut x0, mut y0) = cell_indices_from_pos(from_x, from_y, block_size);
    let (x1, y1) = cell_indices_from_pos(to_x, to_y, block_size);
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        // If we hit a non-walkable (wall) cell, LOS blocked
        if !is_walkable_cell(maze, x0, y0) {
            return false;
        }
        if x0 == x1 && y0 == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
    true
}

// BFS to get the next cell center towards goal; returns center (x,y) of next cell if path found.
fn next_step_bfs(maze: &Maze, from_x: f32, from_y: f32, to_x: f32, to_y: f32, block_size: usize) -> Option<(f32,f32)> {
    let (si,sj) = cell_indices_from_pos(from_x, from_y, block_size);
    let (gi,gj) = cell_indices_from_pos(to_x, to_y, block_size);
    if si == gi && sj == gj { return None; }

    let rows = maze.len();

    let mut q: VecDeque<(isize,isize)> = VecDeque::new();
    // allocate visited and parent with per-row lengths to support non-rectangular mazes
    let mut visited: Vec<Vec<bool>> = Vec::with_capacity(rows);
    let mut parent: Vec<Vec<(isize,isize)>> = Vec::with_capacity(rows);
    for r in maze.iter() {
        visited.push(vec![false; r.len()]);
        parent.push(vec![(-1isize, -1isize); r.len()]);
    }

    if !in_bounds(maze, si, sj) || !in_bounds(maze, gi, gj) { return None; }
    if !is_walkable_cell(maze, gi, gj) { return None; }

    visited[sj as usize][si as usize] = true;
    q.push_back((si,sj));

    let dirs = [(1,0),(-1,0),(0,1),(0,-1)];

    while let Some((ci,cj)) = q.pop_front() {
        if ci == gi && cj == gj { break; }
        for (dx,dy) in dirs.iter() {
            let ni = ci + dx;
            let nj = cj + dy;
            if !in_bounds(maze, ni, nj) { continue; }
            if visited[nj as usize][ni as usize] { continue; }
            if !is_walkable_cell(maze, ni, nj) { continue; }
            visited[nj as usize][ni as usize] = true;
            parent[nj as usize][ni as usize] = (ci,cj);
            q.push_back((ni,nj));
        }
    }

    if !visited[gj as usize][gi as usize] { return None; }

    // reconstruct path from goal to start, stop at the first step
    let mut cur = (gi,gj);
    let mut prev = parent[cur.1 as usize][cur.0 as usize];
    while prev != (-1,-1) && !(prev.0 == si && prev.1 == sj) {
        cur = prev;
        prev = parent[cur.1 as usize][cur.0 as usize];
    }
    // cur now holds the first cell after start
    let center_x = (cur.0 as f32 + 0.5) * block_size as f32;
    let center_y = (cur.1 as f32 + 0.5) * block_size as f32;
    Some((center_x, center_y))
}

pub struct NPC {
    pub pos: Vector2,
    pub speed: f32,
}

impl NPC {
    pub fn new(x: f32, y: f32, speed: f32) -> Self {
        NPC { pos: Vector2::new(x, y), speed }
    }
}

pub fn load_npcs_from_maze(maze: &Maze, block_size: usize) -> Vec<NPC> {
    let mut out = Vec::new();
    for (ry, row) in maze.iter().enumerate() {
        for (rx, &cell) in row.iter().enumerate() {
            if cell == 'R' {
                let cx = (rx as f32 + 0.5) * block_size as f32;
                let cy = (ry as f32 + 0.5) * block_size as f32;
                out.push(NPC::new(cx, cy, 4.5));
            }
        }
    }
    out
}

pub fn update_npcs(npcs: &mut Vec<NPC>, player: &Player, maze: &Maze, block_size: usize) -> bool {
    // return true when any NPC touches the player
    let mut touched = false;
    for npc in npcs.iter_mut() {
        let dir_x = player.pos.x - npc.pos.x;
        let dir_y = player.pos.y - npc.pos.y;
        let len = (dir_x*dir_x + dir_y*dir_y).sqrt();
        // collision threshold (world pixels). If npc gets very close, consider player dead.
        let collision_dist = (block_size as f32) * 0.25; // quarter of cell
        if len <= collision_dist {
            touched = true;
            // continue updating others but mark touched
        }

        if len > 1.0 {
            // If direct LOS to player exists, try moving straight (with sliding)
            if line_of_sight(maze, npc.pos.x, npc.pos.y, player.pos.x, player.pos.y, block_size) {
                let vx = dir_x / len * npc.speed;
                let vy = dir_y / len * npc.speed;
                let nx = npc.pos.x + vx;
                let ny = npc.pos.y + vy;
                if can_move_to(maze, nx, ny, block_size) {
                    npc.pos.x = nx;
                    npc.pos.y = ny;
                    continue;
                }
                // sliding fallback
                if can_move_to(maze, nx, npc.pos.y, block_size) {
                    npc.pos.x = nx;
                }
                if can_move_to(maze, npc.pos.x, ny, block_size) {
                    npc.pos.y = ny;
                }
            } else {
                // No LOS: attempt to step towards next cell along a BFS path
                if let Some((tx,ty)) = next_step_bfs(maze, npc.pos.x, npc.pos.y, player.pos.x, player.pos.y, block_size) {
                    // move toward center of next cell with same speed
                    let dx2 = tx - npc.pos.x;
                    let dy2 = ty - npc.pos.y;
                    let l2 = (dx2*dx2 + dy2*dy2).sqrt().max(0.0001);
                    let vx = dx2 / l2 * npc.speed;
                    let vy = dy2 / l2 * npc.speed;
                    let nx = npc.pos.x + vx;
                    let ny = npc.pos.y + vy;
                    if can_move_to(maze, nx, ny, block_size) {
                        npc.pos.x = nx;
                        npc.pos.y = ny;
                    } else {
                        // as a last resort try axis sliding
                        if can_move_to(maze, nx, npc.pos.y, block_size) {
                            npc.pos.x = nx;
                        }
                        if can_move_to(maze, npc.pos.x, ny, block_size) {
                            npc.pos.y = ny;
                        }
                    }
                }
            }
        }
    }
    touched
}

pub fn render_npcs(framebuffer: &mut Framebuffer, textures: &TextureAtlas, player: &Player, npcs: &Vec<NPC>) {
    let num_rays = framebuffer.width as f32;
    let hh = framebuffer.height as f32 / 2.0;

    for npc in npcs.iter() {
        let cx = npc.pos.x;
        let cy = npc.pos.y;
        let dx = cx - player.pos.x;
        let dy = cy - player.pos.y;
        let dist = (dx*dx + dy*dy).sqrt().max(0.001);
        let ang = dy.atan2(dx);
        let rel_ang = (ang - player.a + std::f32::consts::PI).rem_euclid(2.0*std::f32::consts::PI) - std::f32::consts::PI;
        let half_fov = player.fov / 2.0;
        if rel_ang.abs() > half_fov { continue; }
        let screen_x = ((rel_ang + half_fov) / player.fov) * num_rays;
        let sprite_height = (hh / dist) * 70.0;
        let top = (hh - (sprite_height/2.0)) as isize;
        let bottom = (hh + (sprite_height/2.0)) as isize;
        let sx = screen_x as isize;
        let sprite_screen_w = ((sprite_height * 0.5).max(6.0)) as isize;
        let half_w = (sprite_screen_w / 2).max(1);

        for xoff in -half_w..=half_w {
            let u = (xoff + half_w) as f32 / (sprite_screen_w as f32);
            for y in top.max(0)..bottom.min(framebuffer.height as isize) {
                let v = (y as f32 - top as f32) / (bottom as f32 - top as f32 + 1.0);
                let px = sx + xoff;
                if px >= 0 && px < num_rays as isize {
                    if let Some(col) = textures.sample_npc(u, v) {
                        if col.a > 16 {
                            framebuffer.set_current_color(col);
                            framebuffer.set_pixel(px as u32, y as u32);
                        }
                    } else {
                        framebuffer.set_current_color(Color::new(200,30,30,255));
                        framebuffer.set_pixel(px as u32, y as u32);
                    }
                }
            }
        }
    }
}
