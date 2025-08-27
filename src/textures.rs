// textures.rs

use raylib::prelude::*;
use std::path::Path;
use image::GenericImageView;
use crate::anim::CoinAnimation;

#[derive(Copy, Clone, Debug)]
pub enum TextureKind {
    Wall,
    Pillar,
    DoorClosed,
    DoorOpen,
}

pub struct ImageBuf {
    pub w: u32,
    pub h: u32,
    pub data: Vec<u8>, // RGBA8
}

pub struct TextureAtlas {
    pub wall: Option<ImageBuf>,
    pub pillar: Option<ImageBuf>,
    pub npc: Option<ImageBuf>,
    pub sky: Option<ImageBuf>,
    pub floor: Option<ImageBuf>,
    pub menu: Option<ImageBuf>,
    pub game_over: Option<ImageBuf>,
    pub coin: Option<ImageBuf>,
    pub door_closed: Option<ImageBuf>,
    pub door_open: Option<ImageBuf>,
}

impl TextureAtlas {
    pub fn new() -> Self {
        // Try a few candidate relative paths because the working directory may vary.
        let wall_candidates = [
            "./textures/Textura1_PARED.png",
            "textures/Textura1_PARED.png",
            "../textures/Textura1_PARED.png",
        ];
        let pillar_candidates = [
            "./textures/Textura2_Pilar.png",
            "textures/Textura2_Pilar.png",
            "../textures/Textura2_Pilar.png",
        ];

        let mut wall: Option<ImageBuf> = None;
        for p in wall_candidates.iter() {
            let path = Path::new(p);
            if path.exists() {
                eprintln!("[textures] found wall image at {}", path.display());
                match image::open(path) {
                    Ok(img) => {
                        let img = img.to_rgba8();
                        let (w, h) = img.dimensions();
                        wall = Some(ImageBuf { w, h, data: img.into_raw() });
                        break;
                    }
                    Err(e) => eprintln!("[textures] failed to load {}: {:?}", path.display(), e),
                }
            }
        }

        let mut pillar: Option<ImageBuf> = None;
        for p in pillar_candidates.iter() {
            let path = Path::new(p);
            if path.exists() {
                eprintln!("[textures] found pillar image at {}", path.display());
                match image::open(path) {
                    Ok(img) => {
                        let img = img.to_rgba8();
                        let (w, h) = img.dimensions();
                        let raw = img.into_raw();
                        // debug: print first pixel if available
                        if raw.len() >= 4 {
                            eprintln!(
                                "[textures] pillar dims={}x{} first_rgba={},{},{},{}",
                                w,
                                h,
                                raw[0],
                                raw[1],
                                raw[2],
                                raw[3]
                            );
                        }
                        pillar = Some(ImageBuf { w, h, data: raw });
                        break;
                    }
                    Err(e) => eprintln!("[textures] failed to load {}: {:?}", path.display(), e),
                }
            }
        }

        if wall.is_none() {
            eprintln!("[textures] wall image not found in candidates");
        }
        if pillar.is_none() {
            eprintln!("[textures] pillar image not found in candidates");
        }

        // try NPC sprite
        let npc_candidates = [
            "./textures/Letra _R_ Amenazante en Pixel Art.png",
            "textures/Letra _R_ Amenazante en Pixel Art.png",
            "../textures/Letra _R_ Amenazante en Pixel Art.png",
        ];
        let mut npc: Option<ImageBuf> = None;
        for p in npc_candidates.iter() {
            let path = Path::new(p);
            if path.exists() {
                eprintln!("[textures] found npc sprite at {}", path.display());
                match image::open(path) {
                    Ok(img) => {
                        let img = img.to_rgba8();
                        let (w, h) = img.dimensions();
                        npc = Some(ImageBuf { w, h, data: img.into_raw() });
                        break;
                    }
                    Err(e) => eprintln!("[textures] failed to load {}: {:?}", path.display(), e),
                }
            }
        }

        // try sky texture
        let sky_candidates = [
            "./textures/Textura_Cielo.png",
            "textures/Textura_Cielo.png",
            "../textures/Textura_Cielo.png",
        ];
        let mut sky: Option<ImageBuf> = None;
        for p in sky_candidates.iter() {
            let path = Path::new(p);
            if path.exists() {
                eprintln!("[textures] found sky image at {}", path.display());
                match image::open(path) {
                    Ok(img) => {
                        let img = img.to_rgba8();
                        let (w, h) = img.dimensions();
                        sky = Some(ImageBuf { w, h, data: img.into_raw() });
                        break;
                    }
                    Err(e) => eprintln!("[textures] failed to load {}: {:?}", path.display(), e),
                }
            }
        }

        // try floor texture
        let floor_candidates = [
            "./textures/Textura_Piso.png",
            "textures/Textura_Piso.png",
            "./textures/floor.jpg",
            "textures/floor.jpg",
            "./textures/floor.png",
            "textures/floor.png",
            "../textures/floor.jpg",
        ];
        let mut floor: Option<ImageBuf> = None;
        for p in floor_candidates.iter() {
            let path = Path::new(p);
            if path.exists() {
                eprintln!("[textures] found floor image at {}", path.display());
                match image::open(path) {
                    Ok(img) => {
                        let img = img.to_rgba8();
                        let (w, h) = img.dimensions();
                        floor = Some(ImageBuf { w, h, data: img.into_raw() });
                        break;
                    }
                    Err(e) => eprintln!("[textures] failed to load {}: {:?}", path.display(), e),
                }
            }
        }

        // try menu background texture (user-provided)
        let menu_candidates = [
            "./textures/menu.png",
            "textures/menu.png",
            "./textures/menu_background.png",
            "textures/menu_background.png",
            "../textures/menu.png",
        ];
        let mut menu: Option<ImageBuf> = None;
        for p in menu_candidates.iter() {
            let path = Path::new(p);
            if path.exists() {
                eprintln!("[textures] found menu image at {}", path.display());
                match image::open(path) {
                    Ok(img) => {
                        let img = img.to_rgba8();
                        let (w, h) = img.dimensions();
                        menu = Some(ImageBuf { w, h, data: img.into_raw() });
                        break;
                    }
                    Err(e) => eprintln!("[textures] failed to load {}: {:?}", path.display(), e),
                }
            }
        }

        // try game over texture
        let game_candidates = [
            "./textures/game_over.png",
            "textures/game_over.png",
            "./textures/gameover.png",
            "textures/gameover.png",
            "../textures/game_over.png",
        ];
        let mut game_over: Option<ImageBuf> = None;
        for p in game_candidates.iter() {
            let path = Path::new(p);
            if path.exists() {
                eprintln!("[textures] found game_over image at {}", path.display());
                match image::open(path) {
                    Ok(img) => {
                        let img = img.to_rgba8();
                        let (w, h) = img.dimensions();
                        game_over = Some(ImageBuf { w, h, data: img.into_raw() });
                        break;
                    }
                    Err(e) => eprintln!("[textures] failed to load {}: {:?}", path.display(), e),
                }
            }
        }

        // try coin spritesheet
        let coin_candidates = [
            "./textures/coin_spin_64x64_12f.png",
            "textures/coin_spin_64x64_12f.png",
            "../textures/coin_spin_64x64_12f.png",
        ];
        let mut coin: Option<ImageBuf> = None;
        for p in coin_candidates.iter() {
            let path = Path::new(p);
            if path.exists() {
                eprintln!("[textures] found coin spritesheet at {}", path.display());
                match image::open(path) {
                    Ok(img) => {
                        let img = img.to_rgba8();
                        let (w, h) = img.dimensions();
                        coin = Some(ImageBuf { w, h, data: img.into_raw() });
                        break;
                    }
                    Err(e) => eprintln!("[textures] failed to load {}: {:?}", path.display(), e),
                }
            }
        }

        // Load door textures
        let door_closed_candidates = [
            "./textures/puertacerrada.png",
            "textures/puertacerrada.png",
            "../textures/puertacerrada.png",
        ];
        let mut door_closed: Option<ImageBuf> = None;
        for p in door_closed_candidates.iter() {
            let path = Path::new(p);
            if path.exists() {
                eprintln!("[textures] found door closed texture at {}", path.display());
                match image::open(path) {
                    Ok(img) => {
                        let img = img.to_rgba8();
                        let (w, h) = img.dimensions();
                        door_closed = Some(ImageBuf { w, h, data: img.into_raw() });
                        break;
                    }
                    Err(e) => eprintln!("[textures] failed to load {}: {:?}", path.display(), e),
                }
            }
        }

        let door_open_candidates = [
            "./textures/Puertaabierta.png",
            "textures/Puertaabierta.png", 
            "../textures/Puertaabierta.png",
        ];
        let mut door_open: Option<ImageBuf> = None;
        for p in door_open_candidates.iter() {
            let path = Path::new(p);
            if path.exists() {
                eprintln!("[textures] found door open texture at {}", path.display());
                match image::open(path) {
                    Ok(img) => {
                        let img = img.to_rgba8();
                        let (w, h) = img.dimensions();
                        door_open = Some(ImageBuf { w, h, data: img.into_raw() });
                        break;
                    }
                    Err(e) => eprintln!("[textures] failed to load {}: {:?}", path.display(), e),
                }
            }
        }

    TextureAtlas { wall, pillar, npc, sky, floor, menu, game_over, coin, door_closed, door_open }
    }

    // Sample color from the chosen texture image by normalized u,v in [0,1]
    // If the image isn't loaded, return a procedural fallback color pattern.
    pub fn sample(&self, kind: TextureKind, u: f32, v: f32) -> Color {
        // keep fractional repeat behavior, but sample with bilinear filtering
        let u = u.fract().abs();
        let v = v.fract().abs();

        let img_opt = match kind {
            TextureKind::Wall => &self.wall,
            TextureKind::Pillar => &self.pillar,
            TextureKind::DoorClosed => &self.door_closed,
            TextureKind::DoorOpen => &self.door_open,
        };

        if img_opt.is_none() {
            eprintln!("[textures::sample] warning: requested texture {:?} not loaded", kind);
        }

        if let Some(img) = img_opt {
            if img.data.len() >= 4 {
                // bilinear filtering: compute floating sample coordinates in [0, w-1], [0, h-1]
                let fw = (img.w - 1) as f32;
                let fh = (img.h - 1) as f32;
                let xf = (u * fw).clamp(0.0, fw);
                let yf = (v * fh).clamp(0.0, fh);
                let x0 = xf.floor() as u32;
                let y0 = yf.floor() as u32;
                let x1 = (x0 + 1).min(img.w - 1);
                let y1 = (y0 + 1).min(img.h - 1);
                let sx = xf - x0 as f32;
                let sy = yf - y0 as f32;

                let sample_pixel = |xx: u32, yy: u32| -> (f32,f32,f32,f32) {
                    let idx = ((yy * img.w + xx) * 4) as usize;
                    if idx + 3 < img.data.len() {
                        let r = img.data[idx] as f32 / 255.0;
                        let g = img.data[idx + 1] as f32 / 255.0;
                        let b = img.data[idx + 2] as f32 / 255.0;
                        let a = img.data[idx + 3] as f32 / 255.0;
                        let a = if a == 0.0 { 1.0 } else { a };
                        return (r, g, b, a);
                    }
                    (0.0, 0.0, 0.0, 1.0)
                };

                let (r00,g00,b00,a00) = sample_pixel(x0,y0);
                let (r10,g10,b10,a10) = sample_pixel(x1,y0);
                let (r01,g01,b01,a01) = sample_pixel(x0,y1);
                let (r11,g11,b11,a11) = sample_pixel(x1,y1);

                // lerp horizontally then vertically
                let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;

                let r0 = lerp(r00, r10, sx);
                let g0 = lerp(g00, g10, sx);
                let b0 = lerp(b00, b10, sx);
                let a0 = lerp(a00, a10, sx);

                let r1 = lerp(r01, r11, sx);
                let g1 = lerp(g01, g11, sx);
                let b1 = lerp(b01, b11, sx);
                let a1 = lerp(a01, a11, sx);

                let r = lerp(r0, r1, sy);
                let g = lerp(g0, g1, sy);
                let b = lerp(b0, b1, sy);
                let a = lerp(a0, a1, sy);

                let out_r = (r*255.0) as u8;
                let out_g = (g*255.0) as u8;
                let out_b = (b*255.0) as u8;
                let out_a = (a*255.0) as u8;
                // If the sampled color is pure black, treat it as missing and fall back
                if out_r == 0 && out_g == 0 && out_b == 0 {
                    // fall through to procedural fallback below
                } else {
                    return Color::new(out_r, out_g, out_b, out_a);
                }
            }
        }

        // Procedural fallback: a simple checkerboard
        let checks = 8.0;
        let uu = (u * checks) as i32;
        let vv = (v * checks) as i32;
        if (uu + vv) % 2 == 0 {
            Color::new(200, 180, 160, 255)
        } else {
            Color::new(140, 120, 100, 255)
        }
    }

    pub fn sample_npc(&self, u: f32, v: f32) -> Option<Color> {
        let u = u.fract().abs();
        let v = v.fract().abs();
        if let Some(img) = &self.npc {
            if img.data.len() >= 4 {
                let x = ((u * img.w as f32).clamp(0.0, (img.w - 1) as f32)) as u32;
                let y = ((v * img.h as f32).clamp(0.0, (img.h - 1) as f32)) as u32;
                let idx = ((y * img.w + x) * 4) as usize;
                if idx + 3 < img.data.len() {
                    let r = img.data[idx];
                    let g = img.data[idx + 1];
                    let b = img.data[idx + 2];
                    let a = img.data[idx + 3];
                    return Some(Color::new(r as u8, g as u8, b as u8, a as u8));
                }
            }
        }
        None
    }

    // Sample the sky texture by normalized u (horiz) and v (vert). If missing, return a gradient.
    pub fn sample_sky(&self, u: f32, v: f32) -> Color {
        let u = u.fract().abs();
        let v = v.fract().abs();
        if let Some(img) = &self.sky {
            if img.data.len() >= 4 {
                let x = ((u * img.w as f32).clamp(0.0, (img.w - 1) as f32)) as u32;
                let y = ((v * img.h as f32).clamp(0.0, (img.h - 1) as f32)) as u32;
                let idx = ((y * img.w + x) * 4) as usize;
                if idx + 3 < img.data.len() {
                    let r = img.data[idx];
                    let g = img.data[idx + 1];
                    let b = img.data[idx + 2];
                    let a = img.data[idx + 3];
                    return Color::new(r as u8, g as u8, b as u8, a as u8);
                }
            }
        }
        // fallback: vertical gradient sky
        let top = Color::new(80, 160, 240, 255);
        let bottom = Color::new(160, 200, 240, 255);
        let mix = v;
        let r = (top.r as f32 * (1.0 - mix) + bottom.r as f32 * mix) as u8;
        let g = (top.g as f32 * (1.0 - mix) + bottom.g as f32 * mix) as u8;
        let b = (top.b as f32 * (1.0 - mix) + bottom.b as f32 * mix) as u8;
        Color::new(r, g, b, 255)
    }

    pub fn sample_floor(&self, u: f32, v: f32) -> Color {
        let u = u.fract().abs();
        let v = v.fract().abs();
        if let Some(img) = &self.floor {
            if img.data.len() >= 4 {
                let x = ((u * img.w as f32).clamp(0.0, (img.w - 1) as f32)) as u32;
                let y = ((v * img.h as f32).clamp(0.0, (img.h - 1) as f32)) as u32;
                let idx = ((y * img.w + x) * 4) as usize;
                if idx + 3 < img.data.len() {
                    let r = img.data[idx];
                    let g = img.data[idx + 1];
                    let b = img.data[idx + 2];
                    let a = img.data[idx + 3];
                    return Color::new(r as u8, g as u8, b as u8, a as u8);
                }
            }
        }
        // fallback tiled checker
        let checks = 6.0;
        let uu = (u * checks) as i32;
        let vv = (v * checks) as i32;
        if (uu + vv) % 2 == 0 {
            Color::new(120, 90, 60, 255)
        } else {
            Color::new(100, 80, 60, 255)
        }
    }

    // Sample the menu background texture if available, else return a dark gradient
    pub fn sample_menu(&self, u: f32, v: f32) -> Color {
        let u = u.fract().abs();
        let v = v.fract().abs();
        if let Some(img) = &self.menu {
            if img.data.len() >= 4 {
                let x = ((u * img.w as f32).clamp(0.0, (img.w - 1) as f32)) as u32;
                let y = ((v * img.h as f32).clamp(0.0, (img.h - 1) as f32)) as u32;
                let idx = ((y * img.w + x) * 4) as usize;
                if idx + 3 < img.data.len() {
                    let r = img.data[idx];
                    let g = img.data[idx + 1];
                    let b = img.data[idx + 2];
                    let a = img.data[idx + 3];
                    return Color::new(r as u8, g as u8, b as u8, a as u8);
                }
            }
        }
        // fallback: dark vignette-like gradient
        let top = Color::new(30, 20, 10, 255);
        let bottom = Color::new(10, 10, 10, 255);
        let mix = v;
        let r = (top.r as f32 * (1.0 - mix) + bottom.r as f32 * mix) as u8;
        let g = (top.g as f32 * (1.0 - mix) + bottom.g as f32 * mix) as u8;
        let b = (top.b as f32 * (1.0 - mix) + bottom.b as f32 * mix) as u8;
        Color::new(r, g, b, 255)
    }

    pub fn sample_gameover(&self, u: f32, v: f32) -> Color {
        let u = u.fract().abs();
        let v = v.fract().abs();
        if let Some(img) = &self.game_over {
            if img.data.len() >= 4 {
                let x = ((u * img.w as f32).clamp(0.0, (img.w - 1) as f32)) as u32;
                let y = ((v * img.h as f32).clamp(0.0, (img.h - 1) as f32)) as u32;
                let idx = ((y * img.w + x) * 4) as usize;
                if idx + 3 < img.data.len() {
                    let r = img.data[idx];
                    let g = img.data[idx + 1];
                    let b = img.data[idx + 2];
                    let a = img.data[idx + 3];
                    return Color::new(r as u8, g as u8, b as u8, a as u8);
                }
            }
        }
        // fallback: red vignette
        let top = Color::new(80, 10, 10, 255);
        let bottom = Color::new(20, 10, 10, 255);
        let mix = v;
        let r = (top.r as f32 * (1.0 - mix) + bottom.r as f32 * mix) as u8;
        let g = (top.g as f32 * (1.0 - mix) + bottom.g as f32 * mix) as u8;
        let b = (top.b as f32 * (1.0 - mix) + bottom.b as f32 * mix) as u8;
        Color::new(r, g, b, 255)
    }

    // Sample coin spritesheet with animation using anim module
    // The spritesheet has 12 frames arranged horizontally (64x64 each)
    pub fn sample_coin(&self, u: f32, v: f32, animation_time: f32) -> Option<Color> {
        let u = u.fract().abs();
        let v = v.fract().abs();
        
        if let Some(img) = &self.coin {
            if img.data.len() >= 4 {
                // Get frame info from animation module
                let num_frames = 12;
                let frame_width = img.w / num_frames as u32;
                let frame_height = img.h;
                
                // Get the x offset for the current frame using anim module
                let frame_x_offset = CoinAnimation::get_frame_offset(animation_time, frame_width);
                
                // Sample within the current frame
                let x = ((u * frame_width as f32).clamp(0.0, (frame_width - 1) as f32)) as u32 + frame_x_offset;
                let y = ((v * frame_height as f32).clamp(0.0, (frame_height - 1) as f32)) as u32;
                
                let idx = ((y * img.w + x) * 4) as usize;
                if idx + 3 < img.data.len() {
                    let r = img.data[idx];
                    let g = img.data[idx + 1];
                    let b = img.data[idx + 2];
                    let a = img.data[idx + 3];
                    return Some(Color::new(r as u8, g as u8, b as u8, a as u8));
                }
            }
        }
        None
    }
}
