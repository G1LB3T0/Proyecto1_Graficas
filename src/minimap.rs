// minimap.rs

use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;
use crate::sprite::NPC;
use crate::line::line;
use raylib::prelude::*;
use std::f32::consts::PI;

// Square, smaller and clearer minimap.
pub fn render_minimap(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    scale: usize,
    player: &Player,
    xo: usize,
    yo: usize,
    block_size: usize,
    npcs: &Vec<NPC>,
) {
    let cols = if maze.is_empty() { 1 } else { maze[0].len() };
    let rows = maze.len();
    let max_cells = cols.max(rows).max(1);

    // side length: keep it smaller than previous versions
    let padding = 24;
    let side = (max_cells * scale) as i32 + padding; // square minimap side
    let ox = xo as i32;
    let oy = yo as i32;

    // background rectangle (semi-transparent)
    framebuffer.set_current_color(Color::new(12, 18, 28, 220));
    for x in ox..(ox + side) {
        for y in oy..(oy + side) {
            if x >= 0 && y >= 0 {
                framebuffer.set_pixel(x as u32, y as u32);
            }
        }
    }

    // compute effective cell size to fit square nicely
    let effective_scale = ((side - padding) as f32) / (max_cells as f32);
    let origin_x = ox as f32 + (padding as f32 / 2.0);
    let origin_y = oy as f32 + (padding as f32 / 2.0);

    // draw cells with clear contrast: corridors light, walls dark
    for (r, row) in maze.iter().enumerate() {
        for (c, &cell) in row.iter().enumerate() {
            let sx = origin_x + (c as f32) * effective_scale;
            let sy = origin_y + (r as f32) * effective_scale;
            let ix = sx as i32;
            let iy = sy as i32;

            // choose color: corridors (space) lighter, walls darker
            let color = match cell {
                '+' | '-' | '|' => Color::new(40, 40, 50, 255), // wall
                'g' => Color::GREEN,
                // treat R as corridor base so the red glyph stands out
                'R' => Color::new(230, 230, 230, 255),
                _ => Color::new(230, 230, 230, 255), // corridor/floor
            };

            framebuffer.set_current_color(color);
            // draw a tight square for the cell with a 1px gap so corridors read clearly
            let cell_px = (effective_scale * 0.9).max(1.0) as i32; // shrink a bit for gutter
            let half = cell_px / 2;
            let center_x = ix + (effective_scale as i32) / 2;
            let center_y = iy + (effective_scale as i32) / 2;
            for oxi in -half..=half {
                for oyi in -half..=half {
                    let px = center_x + oxi;
                    let py = center_y + oyi;
                    if px >= 0 && py >= 0 {
                        framebuffer.set_pixel(px as u32, py as u32);
                    }
                }
            }

            // if this cell contains an 'R', draw a small red 'R' glyph centered in the cell
            if cell == 'R' {
                // 3x5 stylized R bitmap (1 = pixel)
                let glyph: [[u8; 3]; 5] = [
                    [1,1,1],
                    [1,0,1],
                    [1,1,1],
                    [1,0,1],
                    [1,0,1],
                ];
                let glyph_w = 3;
                let glyph_h = 5;
                let scale_g = (effective_scale * 0.18).max(1.0) as i32; // scale to cell
                let gx0 = center_x - ((glyph_w*scale_g)/2);
                let gy0 = center_y - ((glyph_h*scale_g)/2);
                framebuffer.set_current_color(Color::new(200, 30, 30, 255));
                for gy in 0..glyph_h {
                    for gx in 0..glyph_w {
                        if glyph[gy as usize][gx as usize] == 1 {
                            // draw scaled block
                            for sxg in 0..scale_g {
                                for syg in 0..scale_g {
                                    let px = gx0 + gx*scale_g + sxg;
                                    let py = gy0 + gy*scale_g + syg;
                                    if px >= 0 && py >= 0 {
                                        framebuffer.set_pixel(px as u32, py as u32);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // draw grid lines (subtle) to make corridors and cells clearer
    framebuffer.set_current_color(Color::new(20, 20, 28, 160));
    for i in 0..=max_cells {
        let x = origin_x as i32 + (i as f32 * effective_scale) as i32;
        for y in oy..(oy + side) {
            if x >= 0 && y >= 0 {
                framebuffer.set_pixel(x as u32, y as u32);
            }
        }
        let y = origin_y as i32 + (i as f32 * effective_scale) as i32;
        for x in ox..(ox + side) {
            if x >= 0 && y >= 0 {
                framebuffer.set_pixel(x as u32, y as u32);
            }
        }
    }

    // border
    framebuffer.set_current_color(Color::new(255, 255, 255, 90));
    for x in ox..(ox + side) {
        framebuffer.set_pixel(x as u32, oy as u32);
        framebuffer.set_pixel(x as u32, (oy + side - 1) as u32);
    }
    for y in oy..(oy + side) {
        framebuffer.set_pixel(ox as u32, y as u32);
        framebuffer.set_pixel((ox + side - 1) as u32, y as u32);
    }

    // draw player as filled circle + short direction line (bigger and clearer)
    let player_cell_x = player.pos.x / block_size as f32;
    let player_cell_y = player.pos.y / block_size as f32;
    let px_f = origin_x + player_cell_x * effective_scale + (effective_scale * 0.5);
    let py_f = origin_y + player_cell_y * effective_scale + (effective_scale * 0.5);
    let px_i = px_f as i32;
    let py_i = py_f as i32;

    // direction line
    framebuffer.set_current_color(Color::YELLOW);
    let dir_len = (effective_scale * 1.5).max(6.0);
    let ex = px_f + player.a.cos() * dir_len;
    let ey = py_f + player.a.sin() * dir_len;
    let steps = (dir_len * 1.2) as i32;
    for t in 0..=steps {
        let frac = t as f32 / steps as f32;
        let lx = px_f + (ex - px_f) * frac;
        let ly = py_f + (ey - py_f) * frac;
        let ix = lx as i32;
        let iy = ly as i32;
        if ix >= 0 && iy >= 0 {
            framebuffer.set_pixel(ix as u32, iy as u32);
        }
    }

    // filled player circle
    framebuffer.set_current_color(Color::ORANGE);
    let r_player = ((effective_scale * 0.4).max(2.0)) as i32;
    for dx in -r_player..=r_player {
        for dy in -r_player..=r_player {
            if dx*dx + dy*dy <= r_player*r_player {
                let x = px_i + dx;
                let y = py_i + dy;
                if x >= 0 && y >= 0 {
                    framebuffer.set_pixel(x as u32, y as u32);
                }
            }
        }
    }

    // draw NPCs dynamic positions (small red dots)
    framebuffer.set_current_color(Color::RED);
    for npc in npcs.iter() {
        let nx = npc.pos.x / block_size as f32;
        let ny = npc.pos.y / block_size as f32;
        let nxf = origin_x + nx * effective_scale + (effective_scale * 0.5);
        let nyf = origin_y + ny * effective_scale + (effective_scale * 0.5);
        let nxi = nxf as i32;
        let nyi = nyf as i32;
        // small square
        for oxi in -1..=1 {
            for oyi in -1..=1 {
                let px = nxi + oxi;
                let py = nyi + oyi;
                if px >= 0 && py >= 0 {
                    framebuffer.set_pixel(px as u32, py as u32);
                }
            }
        }
    }

    // faint FOV rays (subtle)
    framebuffer.set_current_color(Color::new(255, 210, 120, 120));
    let fov_rays = 8;
    for i in 0..fov_rays {
        let t = if fov_rays > 1 { i as f32 / (fov_rays - 1) as f32 } else { 0.5 };
        let a = player.a - (player.fov / 2.0) + player.fov * t;
        let mut d = 0.0_f32;
        while d < (side as f32) {
            let wx = player.pos.x + a.cos() * d;
            let wy = player.pos.y + a.sin() * d;
            let sx = origin_x + wx / block_size as f32 * effective_scale + (effective_scale * 0.5);
            let sy = origin_y + wy / block_size as f32 * effective_scale + (effective_scale * 0.5);
            let ix = sx as i32;
            let iy = sy as i32;
            if ix >= ox && ix < ox + side && iy >= oy && iy < oy + side {
                framebuffer.set_pixel(ix as u32, iy as u32);
            } else {
                break;
            }
            d += 6.0;
        }
    }
}
