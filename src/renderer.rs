// renderer.rs - clean implementation
#![allow(dead_code)]

use raylib::prelude::*;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;
use crate::caster::cast_ray;
use crate::textures::{TextureAtlas, TextureKind};
use crate::sprite::{NPC, Coin};
use crate::anim::CoinAnimation;
use std::f32::consts::PI;

fn cell_to_color(cell: char) -> Color {
    match cell {
        '+' => Color::BLUEVIOLET,
        '-' => Color::VIOLET,
        '|' => Color::VIOLET,
        'g' => Color::GREEN,
        _ => Color::WHITE,
    }
}

fn draw_cell(
    framebuffer: &mut Framebuffer,
    xo: usize,
    yo: usize,
    block_size: usize,
    cell: char,
) {
    if cell == ' ' || cell == 'C' { return; } // 'C' should be empty space for coins
    let color = cell_to_color(cell);
    framebuffer.set_current_color(color);
    for x in xo..xo + block_size {
        for y in yo..yo + block_size {
            framebuffer.set_pixel(x as u32, y as u32);
        }
    }
}

pub fn render_maze(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    block_size: usize,
    player: &Player,
) {
    for (row_index, row) in maze.iter().enumerate() {
        for (col_index, &cell) in row.iter().enumerate() {
            let xo = col_index * block_size;
            let yo = row_index * block_size;
            draw_cell(framebuffer, xo, yo, block_size, cell);
        }
    }
    framebuffer.set_current_color(Color::WHITESMOKE);
    // debug: draw a few rays to visualize
    for i in 0..5 {
        let t = i as f32 / 5.0;
        let a = player.a - (player.fov / 2.0) + (player.fov * t);
        cast_ray(framebuffer, &maze, &player, a, block_size, true);
    }
}

pub fn render_world(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    block_size: usize,
    player: &Player,
    textures: &TextureAtlas,
    npcs: &Vec<NPC>,
    coins: &Vec<Coin>,
    column_step: usize,
) {
    // Render using coarse columns to reduce the number of rays (improves FPS).
    // column_step controls how many horizontal pixels share the same ray.
    let column_step = column_step.max(1);
    let num_rays = ((framebuffer.width as usize) + column_step - 1) / column_step;
    let hh = framebuffer.height as f32 / 2.0;

    // depth buffer per column for sprite occlusion
    let mut depth_buffer = vec![f32::INFINITY; num_rays];

    // render walls and fill depth buffer (one ray per COLUMN_STEP pixels)
    for i in 0..num_rays {
        let screen_x = i * column_step;
        let ix = screen_x as u32;
        let t = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * t);
        // sky: sample based on ray angle (u)
        let sky_u = (a / (2.0 * PI)).rem_euclid(1.0);
        let intersect = cast_ray(framebuffer, &maze, &player, a, block_size, false);

        // Correct fish-eye: compute angular difference and use cos to get perpendicular distance
        let distance = intersect.distance.max(0.0001);
        let mut angle_diff = (a - player.a).rem_euclid(2.0 * PI);
        if angle_diff > PI { angle_diff -= 2.0 * PI; }
        let perp_dist = (distance * angle_diff.cos()).abs().max(0.0001);
        depth_buffer[i] = perp_dist;
        let stake_h = (hh / perp_dist) * 70.0;

        let mut top = (hh - stake_h / 2.0) as isize;
        let mut bottom = (hh + stake_h / 2.0) as isize;
        if top < 0 { top = 0 }
        if bottom as u32 >= framebuffer.height { bottom = framebuffer.height as isize - 1 }

        // compute texture coordinate u using hit position
            // compute texture coordinate u using hit position and the side the ray hit
            // side == 0 means an x-side (vertical wall), so u should be hit_y fraction
            // side == 1 means a y-side (horizontal wall), so u should be hit_x fraction
            let u = {
                let bx = block_size as f32;
                let frac_x = (intersect.hit_x / bx).fract();
                let frac_y = (intersect.hit_y / bx).fract();
                if intersect.side == 0 { frac_y } else { frac_x }
            };

        let kind = match intersect.impact { '+' => TextureKind::Pillar, _ => TextureKind::Wall };

        // draw sky above the top of the wall column (same color across the COLUMN_STEP width)
        for y in 0..top.max(0) as isize {
            let v = (y as f32) / (hh); // top..hh maps to 0..1
            let col = textures.sample_sky(sky_u, v);
            framebuffer.set_current_color(col);
            for xoff in 0..column_step {
                let px = ix + xoff as u32;
                if px >= framebuffer.width { break }
                framebuffer.set_pixel(px, y as u32);
            }
        }

        // draw wall column across COLUMN_STEP width
        for y in top..=bottom {
            // screen-space fraction along the wall column
            let v_frac = (y as f32 - top as f32) / (bottom as f32 - top as f32 + 1.0);
            // get the texture pixel height for this kind, default to 32 if missing
            let tex_h_pixels: u32 = match kind {
                TextureKind::Wall => textures.wall.as_ref().map(|i| i.h).unwrap_or(32),
                TextureKind::Pillar => textures.pillar.as_ref().map(|i| i.h).unwrap_or(32),
            };
            // Tile the texture according to world-space wall height (block_size) so the
            // texture repeats per block remain constant regardless of camera distance.
            let repeats_world = (block_size as f32) / (tex_h_pixels as f32);
            // clamp extreme values to avoid too many tiny tiles or excessive stretching
            let repeats = repeats_world.clamp(0.25, 4.0);
            let v_param = v_frac * repeats;
            let col = textures.sample(kind, u, v_param);
            framebuffer.set_current_color(col);
            for xoff in 0..column_step {
                let px = ix + xoff as u32;
                if px >= framebuffer.width { break }
                framebuffer.set_pixel(px, y as u32);
            }
        }

        // draw floor below the wall column - fill COLUMN_STEP width
        let floor_base = Color::new(90, 30, 30, 255);
        for y in (bottom+1)..=(framebuffer.height as isize - 1) {
            framebuffer.set_current_color(floor_base);
            for xoff in 0..column_step {
                let px = ix + xoff as u32;
                if px >= framebuffer.width { break }
                framebuffer.set_pixel(px, y as u32);
            }
        }
    }

    // render sprites with occlusion using column depth buffer
    for npc in npcs.iter() {
        let dx = npc.pos.x - player.pos.x;
        let dy = npc.pos.y - player.pos.y;
        let dist = (dx*dx + dy*dy).sqrt().max(0.001);
        let ang = dy.atan2(dx);
        let rel = (ang - player.a + std::f32::consts::PI).rem_euclid(2.0*std::f32::consts::PI) - std::f32::consts::PI;
        if rel.abs() > player.fov / 2.0 { continue }

    // screen_x in pixels (full framebuffer width), then we will map pixel -> column index
    let screen_x = ((rel + player.fov/2.0) / player.fov) * framebuffer.width as f32;
        let sprite_h = (hh / dist) * 70.0;
        let top = (hh - sprite_h/2.0) as isize;
        let bottom = (hh + sprite_h/2.0) as isize;
        let sx = screen_x as isize;
        let w = ((sprite_h * 0.5).max(3.0)) as isize;
        let half = (w / 2).max(1);

        for xoff in -half..=half {
            let px = sx + xoff;
            if px < 0 { continue }
            // map pixel x to depth_buffer column index (integer division by COLUMN_STEP)
            let col_idx = (px as usize) / column_step;
            if col_idx >= num_rays { continue }
            if dist > depth_buffer[col_idx] - 1.0 { continue }

            for y in top.max(0)..=bottom.min(framebuffer.height as isize - 1) {
                let v = (y as f32 - top as f32) / (bottom as f32 - top as f32 + 1.0);
                let u = (xoff + half) as f32 / (w as f32);
                if let Some(col) = textures.sample_npc(u, v) {
                    if col.a > 16 {
                        framebuffer.set_current_color(col);
                        framebuffer.set_pixel(px as u32, y as u32);
                    }
                }
            }
        }
    }

    // render coins with occlusion using column depth buffer
    for coin in coins.iter() {
        if coin.collected { continue; }
        
        let dx = coin.pos.x - player.pos.x;
        let dy = coin.pos.y - player.pos.y;
        let dist = (dx*dx + dy*dy).sqrt().max(0.001);
        let ang = dy.atan2(dx);
        let rel = (ang - player.a + std::f32::consts::PI).rem_euclid(2.0*std::f32::consts::PI) - std::f32::consts::PI;
        if rel.abs() > player.fov / 2.0 { continue }

        // screen_x in pixels (full framebuffer width), then we will map pixel -> column index
        let screen_x = ((rel + player.fov/2.0) / player.fov) * framebuffer.width as f32;
        
        // Add floating motion using anim module
        let float_offset = CoinAnimation::get_float_offset(coin.animation_time);
        let sprite_h = (hh / dist) * 60.0; // slightly smaller than NPCs
        let top = (hh - sprite_h/2.0 + float_offset) as isize;
        let bottom = (hh + sprite_h/2.0 + float_offset) as isize;
        let sx = screen_x as isize;
        let w = ((sprite_h * 0.8).max(4.0)) as isize; // slightly wider
        let half = (w / 2).max(1);

        for xoff in -half..=half {
            let px = sx + xoff;
            if px < 0 { continue }
            // map pixel x to depth_buffer column index (integer division by COLUMN_STEP)
            let col_idx = (px as usize) / column_step;
            if col_idx >= num_rays { continue }
            if dist > depth_buffer[col_idx] - 1.0 { continue } // occlusion check

            for y in top.max(0)..=bottom.min(framebuffer.height as isize - 1) {
                let v = (y as f32 - top as f32) / (bottom as f32 - top as f32 + 1.0);
                let u = (xoff + half) as f32 / (w as f32);
                if let Some(col) = textures.sample_coin(u, v, coin.animation_time) {
                    if col.a > 64 { // higher alpha threshold for better visibility
                        framebuffer.set_current_color(col);
                        framebuffer.set_pixel(px as u32, y as u32);
                    }
                }
            }
        }
    }
}
