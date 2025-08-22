// renderer.rs - clean implementation
#![allow(dead_code)]

use raylib::prelude::*;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;
use crate::caster::cast_ray;
use crate::textures::{TextureAtlas, TextureKind};
use crate::sprite::NPC;

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
    if cell == ' ' { return; }
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
) {
    let num_rays = framebuffer.width as usize;
    let hh = framebuffer.height as f32 / 2.0;

    // depth buffer per column for sprite occlusion
    let mut depth_buffer = vec![f32::INFINITY; num_rays];

    // render walls and fill depth buffer
    for i in 0..num_rays {
        let ix = i as u32;
        let t = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * t);
    // sky: sample based on ray angle (u) and vertical v per pixel later
    let sky_u = (a / (2.0 * std::f32::consts::PI)).rem_euclid(1.0);
        let intersect = cast_ray(framebuffer, &maze, &player, a, block_size, false);

        let distance = intersect.distance.max(0.0001);
        depth_buffer[i] = distance;
        let stake_h = (hh / distance) * 70.0;

        let mut top = (hh - stake_h / 2.0) as isize;
        let mut bottom = (hh + stake_h / 2.0) as isize;
        if top < 0 { top = 0 }
        if bottom as u32 >= framebuffer.height { bottom = framebuffer.height as isize - 1 }

        // compute texture coordinate u using hit position
        let u = {
            let bx = block_size as f32;
            let frac_x = (intersect.hit_x / bx).fract();
            let frac_y = (intersect.hit_y / bx).fract();
            let dist_x = frac_x.min(1.0 - frac_x);
            let dist_y = frac_y.min(1.0 - frac_y);
            if dist_x < dist_y { frac_y } else { frac_x }
        };

        let kind = match intersect.impact { '+' => TextureKind::Pillar, _ => TextureKind::Wall };

        // draw sky above the top of the wall column
        for y in 0..top.max(0) as isize {
            let v = (y as f32) / (hh); // top..hh maps to 0..1
            let col = textures.sample_sky(sky_u, v);
            framebuffer.set_current_color(col);
            framebuffer.set_pixel(ix, y as u32);
        }

        for y in top..=bottom {
            let v = (y as f32 - top as f32) / (bottom as f32 - top as f32 + 1.0);
            let col = textures.sample(kind, u, v);
            framebuffer.set_current_color(col);
            framebuffer.set_pixel(ix, y as u32);
        }

        // draw floor below the wall column - simple subdued red (reduced intensity)
        // This avoids per-pixel texture sampling which was causing stretch and FPS drop.
        let floor_base = Color::new(90, 30, 30, 255);
        for y in (bottom+1)..=(framebuffer.height as isize - 1) {
            framebuffer.set_current_color(floor_base);
            framebuffer.set_pixel(ix, y as u32);
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

        let screen_x = ((rel + player.fov/2.0) / player.fov) * num_rays as f32;
        let sprite_h = (hh / dist) * 70.0;
        let top = (hh - sprite_h/2.0) as isize;
        let bottom = (hh + sprite_h/2.0) as isize;
        let sx = screen_x as isize;
        let w = ((sprite_h * 0.5).max(3.0)) as isize;
        let half = (w / 2).max(1);

        for xoff in -half..=half {
            let px = sx + xoff;
            if px < 0 { continue }
            let col_idx = px as usize;
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
}
