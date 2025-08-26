// caster.rs

use raylib::color::Color;

use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;

pub struct Intersect {
  pub distance: f32,
  pub impact: char,
  pub hit_x: f32,
  pub hit_y: f32,
  pub side: u8, // 0 = vertical (x-side), 1 = horizontal (y-side)
}

pub fn cast_ray(
  _framebuffer: &mut Framebuffer,
  maze: &Maze,
  player: &Player,
  a: f32,
  block_size: usize,
  _draw_line: bool,
) -> Intersect {
  // Use DDA (grid-based) raycasting for performance.
  // Work in cell coordinates (each cell = 1.0); convert player position accordingly.
  let pos_x = player.pos.x / block_size as f32;
  let pos_y = player.pos.y / block_size as f32;
  let ray_dir_x = a.cos();
  let ray_dir_y = a.sin();

  // current map cell
  let mut map_x = pos_x.floor() as isize;
  let mut map_y = pos_y.floor() as isize;

  // length of ray from one x or y side to next x or y side
  let delta_dist_x = if ray_dir_x == 0.0 { f32::INFINITY } else { 1.0 / ray_dir_x.abs() };
  let delta_dist_y = if ray_dir_y == 0.0 { f32::INFINITY } else { 1.0 / ray_dir_y.abs() };

  // step direction and initial sideDist
  let step_x: isize;
  let step_y: isize;
  let mut side_dist_x: f32;
  let mut side_dist_y: f32;

  if ray_dir_x < 0.0 {
    step_x = -1;
    side_dist_x = (pos_x - map_x as f32) * delta_dist_x;
  } else {
    step_x = 1;
    side_dist_x = (map_x as f32 + 1.0 - pos_x) * delta_dist_x;
  }
  if ray_dir_y < 0.0 {
    step_y = -1;
    side_dist_y = (pos_y - map_y as f32) * delta_dist_y;
  } else {
    step_y = 1;
    side_dist_y = (map_y as f32 + 1.0 - pos_y) * delta_dist_y;
  }

  // perform DDA
  let mut hit = false;
  let mut side = 0; // 0 = hit on x-side (vertical wall), 1 = y-side (horizontal wall)
  let max_steps = 2000; // guard
  for _ in 0..max_steps {
    if side_dist_x < side_dist_y {
      side_dist_x += delta_dist_x;
      map_x += step_x;
      side = 0;
    } else {
      side_dist_y += delta_dist_y;
      map_y += step_y;
      side = 1;
    }

    if map_y < 0 || map_x < 0 { break; }
    if (map_y as usize) < maze.len() && (map_x as usize) < maze[map_y as usize].len() {
      // treat 'R' as sprite (non-blocking) so rays pass through
      let cell = maze[map_y as usize][map_x as usize];
      if cell != ' ' && cell != 'R' {
        hit = true;
        break;
      }
    } else {
      // out of bounds - treat as no hit
      break;
    }
  }

  if hit {
    // compute perpendicular distance using accumulated side distances (avoids direct division by ray_dir)
    let perp_dist = if side == 0 {
      side_dist_x - delta_dist_x
    } else {
      side_dist_y - delta_dist_y
    };

    // convert to world units (cell units -> world units)
    let distance = perp_dist * block_size as f32;
    let hit_x = player.pos.x + distance * ray_dir_x;
    let hit_y = player.pos.y + distance * ray_dir_y;

    let impact = maze[map_y as usize][map_x as usize];
  return Intersect { distance, impact, hit_x, hit_y, side: side as u8 };
  }

  // fallback: return large distance
  Intersect { distance: 2000.0, impact: ' ', hit_x: player.pos.x, hit_y: player.pos.y, side: 0 }
}
