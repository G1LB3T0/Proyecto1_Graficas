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
) {
    if maze.is_empty() { return; }

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
    let cols = maze[0].len();

    // background for minimap (semi-transparent dark)
    draw_filled_rect(fb, xo as isize - 4, yo as isize - 4, cols * scale + 8, rows * scale + 8, Color::new(0,0,0,160));

    // draw cells
    for (ry, row) in maze.iter().enumerate() {
        for (rx, &cell) in row.iter().enumerate() {
            let x = xo as isize + (rx * scale) as isize;
            let y = yo as isize + (ry * scale) as isize;
            let col = match cell {
                ' ' => Color::new(200,200,200,220), // floor
                '+' | '|' | '-' => Color::new(40,40,40,255), // walls
                'g' => Color::GREEN,
                'R' => Color::new(180,100,100,255),
                _ => Color::new(140,140,140,220),
            };
            draw_filled_rect(fb, x, y, scale, scale, col);
            // draw cell border to improve readability
            fb.set_current_color(Color::BLACK);
            // top and left borders (cheap)
            for i in 0..scale {
                if x as isize >= 0 {
                    if y as isize >= 0 && x as isize >= 0 && (x as u32) < fb.width && (y as u32 + i as u32) < fb.height {
                        fb.set_pixel(x as u32, y as u32 + i as u32);
                    }
                }
                if y as isize >= 0 {
                    if x as isize >= 0 && (x as u32 + i as u32) < fb.width && (y as u32) < fb.height {
                        fb.set_pixel(x as u32 + i as u32, y as u32);
                    }
                }
            }
        }
    }

    // draw NPCs as small red squares
    for npc in npcs.iter() {
        let mx = (npc.pos.x / block_size as f32) * scale as f32 + xo as f32;
        let my = (npc.pos.y / block_size as f32) * scale as f32 + yo as f32;
        let cx = mx.round() as isize;
        let cy = my.round() as isize;
        draw_filled_rect(fb, cx - 1, cy - 1, 3, 3, Color::RED);
    }

    // draw player as blue dot and a directional line
    let px_f = (player.pos.x / block_size as f32) * scale as f32 + xo as f32;
    let py_f = (player.pos.y / block_size as f32) * scale as f32 + yo as f32;
    let px = px_f.round() as isize;
    let py = py_f.round() as isize;
    draw_filled_rect(fb, px - 1, py - 1, 3, 3, Color::SKYBLUE);

    // draw orientation line (length ~ 2 cells)
    let len = (scale as f32) * 2.0;
    let ex = px_f + player.a.cos() * len;
    let ey = py_f + player.a.sin() * len;

    // simple Bresenham-like line (floating approximation)
    let dx = ex - px_f;
    let dy = ey - py_f;
    let steps = dx.abs().max(dy.abs()).ceil() as i32;
    fb.set_current_color(Color::YELLOW);
    if steps > 0 {
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let lx = px_f + dx * t;
            let ly = py_f + dy * t;
            if lx < 0.0 || ly < 0.0 { continue; }
            let lxi = lx.round() as isize;
            let lyi = ly.round() as isize;
            if lxi < 0 || lyi < 0 { continue; }
            if (lxi as u32) >= fb.width || (lyi as u32) >= fb.height { continue; }
            fb.set_pixel(lxi as u32, lyi as u32);
        }
    }
}
