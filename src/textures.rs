// textures.rs

use raylib::prelude::*;
use std::path::Path;
use image::GenericImageView;

#[derive(Copy, Clone)]
pub enum TextureKind {
    Wall,
    Pillar,
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

    TextureAtlas { wall, pillar, npc, sky, floor }
    }

    // Sample color from the chosen texture image by normalized u,v in [0,1]
    // If the image isn't loaded, return a procedural fallback color pattern.
    pub fn sample(&self, kind: TextureKind, u: f32, v: f32) -> Color {
        let u = u.fract().abs();
        let v = v.fract().abs();

        let img_opt = match kind {
            TextureKind::Wall => &self.wall,
            TextureKind::Pillar => &self.pillar,
        };

        if let Some(img) = img_opt {
            if img.data.len() >= 4 {
                let x = ((u * img.w as f32).clamp(0.0, (img.w - 1) as f32)) as u32;
                let y = ((v * img.h as f32).clamp(0.0, (img.h - 1) as f32)) as u32;
                let idx = ((y * img.w + x) * 4) as usize;
                if idx + 3 < img.data.len() {
                    let r = img.data[idx];
                    let g = img.data[idx + 1];
                    let b = img.data[idx + 2];
                    let mut a = img.data[idx + 3];
                        // if image has fully transparent pixels, treat them as opaque so they render
                        if a == 0 { a = 255; }
                        return Color::new(r as u8, g as u8, b as u8, a as u8);
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
}
