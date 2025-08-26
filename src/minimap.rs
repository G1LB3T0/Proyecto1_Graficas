use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;
use crate::sprite::NPC;
use raylib::prelude::Color;

// Render a simple top-left minimap into the framebuffer.
// - `scale` is pixels per maze cell in the minimap.
// - `xo`, `yo` are pixel offsets inside the framebuffer where the minimap origin is drawn.
// - `block_size` is the world pixels per maze cell (used to convert world coords -> maze cells).
pub fn render_minimap(
    fb: &mut Framebuffer,
    maze: &Maze,
    scale: usize,
    player: &Player,
    xo: usize,
    yo: usize,
    block_size: usize,
    npcs: &Vec<NPC>,
    discovered: &mut Vec<Vec<bool>>,
) {
    if maze.is_empty() { return; }
    // ensure discovered grid matches maze dimensions
    if discovered.len() != maze.len() || discovered.iter().zip(maze.iter()).any(|(drow, mrow)| drow.len() != mrow.len()) {
        *discovered = maze.iter().map(|r| vec![false; r.len()]).collect();
    }
    // helper to clip and draw a filled rect in framebuffer
    let draw_filled_rect = |fb: &mut Framebuffer, x: isize, y: isize, w: usize, h: usize, col: Color| {
        fb.set_current_color(col);
        for iy in 0..h {
            let py = y + iy as isize;
            if py < 0 { continue; }
            for ix in 0..w {
                let px = x + ix as isize;
                if px < 0 { continue; }
                // clip to framebuffer bounds
                if (px as u32) >= fb.width || (py as u32) >= fb.height { continue; }
                fb.set_pixel(px as u32, py as u32);
            }
        }
    };

    let rows = maze.len();
    let max_cols = maze.iter().map(|r| r.len()).max().unwrap_or(0);

    // reveal cells around player (fog-of-war). radius in cells
    let pi = (player.pos.x / block_size as f32).floor() as isize;
    let pj = (player.pos.y / block_size as f32).floor() as isize;
    let reveal_radius: isize = 2; // adjust to reveal more/less
    for dy in -reveal_radius..=reveal_radius {
        for dx in -reveal_radius..=reveal_radius {
            let xi = pi + dx;
            let yj = pj + dy;
            if yj >= 0 && (yj as usize) < discovered.len() {
                if xi >= 0 && (xi as usize) < discovered[yj as usize].len() {
                    discovered[yj as usize][xi as usize] = true;
                }
            }
        }
    }

    // background for minimap (semi-transparent dark) with a crisp outer border sized to widest row
    draw_filled_rect(fb, xo as isize - 6, yo as isize - 6, max_cols * scale + 12, rows * scale + 12, Color::new(8,8,16,200));
    // outer border
    fb.set_current_color(Color::new(220,220,220,200));
    // top border
    for x in (xo as isize - 6)..(xo as isize - 6 + (max_cols * scale + 12) as isize) {
        if x >= 0 && (yo as isize - 6) >= 0 && (x as u32) < fb.width && ((yo as isize - 6) as u32) < fb.height {
            fb.set_pixel(x as u32, (yo as isize - 6) as u32);
        }
    }
    // left border
    for y in (yo as isize - 6)..(yo as isize - 6 + (rows * scale + 12) as isize) {
        if y >= 0 && (xo as isize - 6) >= 0 && (y as u32) < fb.height && ((xo as isize - 6) as u32) < fb.width {
            fb.set_pixel((xo as isize - 6) as u32, y as u32);
        }
    }

    // draw cells with subtle colors and light grid lines
    for (ry, row) in maze.iter().enumerate() {
        for (rx, &cell) in row.iter().enumerate() {
            let x = xo as isize + (rx * scale) as isize;
            let y = yo as isize + (ry * scale) as isize;
            let discovered_cell = discovered.get(ry).and_then(|r| r.get(rx)).copied().unwrap_or(false);
            if !discovered_cell {
                // draw fog for undiscovered cells
                draw_filled_rect(fb, x, y, scale, scale, Color::new(10,10,20,220));
                continue;
            }
            let col = match cell {
                ' ' => Color::new(170,170,180,200), // floor (slightly bluish)
                '+' | '|' | '-' => Color::new(32,32,48,255), // walls dark
                'g' => Color::new(80,160,80,255),
                'R' => Color::new(180,100,100,255),
                _ => Color::new(140,140,140,200),
            };
            draw_filled_rect(fb, x, y, scale, scale, col);
            // subtle grid line on bottom and right edges
            fb.set_current_color(Color::new(20,20,30,120));
            if (y as isize + scale as isize) >= 0 {
                for gx in 0..scale {
                    let px = x + gx as isize;
                    let py = y + scale as isize - 1;
                    if px >= 0 && py >= 0 && (px as u32) < fb.width && (py as u32) < fb.height {
                        fb.set_pixel(px as u32, py as u32);
                    }
                }
            }
            if (x as isize + scale as isize) >= 0 {
                for gy in 0..scale {
                    let px = x + scale as isize - 1;
                    let py = y + gy as isize;
                    if px >= 0 && py >= 0 && (px as u32) < fb.width && (py as u32) < fb.height {
                        fb.set_pixel(px as u32, py as u32);
                    }
                }
            }
        }
    }

    // draw NPCs as small red squares only if their cell was discovered
    for npc in npcs.iter() {
        let cx_cell = (npc.pos.x / block_size as f32).floor() as isize;
        let cy_cell = (npc.pos.y / block_size as f32).floor() as isize;
        if cy_cell < 0 || cx_cell < 0 { continue; }
        if (cy_cell as usize) >= discovered.len() { continue; }
        if (cx_cell as usize) >= discovered[cy_cell as usize].len() { continue; }
        if !discovered[cy_cell as usize][cx_cell as usize] { continue; }
        let mx = (npc.pos.x / block_size as f32) * scale as f32 + xo as f32;
        let my = (npc.pos.y / block_size as f32) * scale as f32 + yo as f32;
        let cx = mx.round() as isize;
        let cy = my.round() as isize;
        draw_filled_rect(fb, cx - 1, cy - 1, 4, 4, Color::RED);
    }

    // draw player as blue dot (no direction ray)
    let px_f = (player.pos.x / block_size as f32) * scale as f32 + xo as f32;
    let py_f = (player.pos.y / block_size as f32) * scale as f32 + yo as f32;
    let px = px_f.round() as isize;
    let py = py_f.round() as isize;
    draw_filled_rect(fb, px - 1, py - 1, 4, 4, Color::SKYBLUE);
}
