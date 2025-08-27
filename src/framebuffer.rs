// framebuffer.rs

use raylib::prelude::*;

pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub color_buffer: Image,
    background_color: Color,
    current_color: Color,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let color_buffer = Image::gen_image_color(width as i32, height as i32, Color::BLACK);
        Framebuffer {
            width,
            height,
            color_buffer,
            background_color: Color::BLACK,
            current_color: Color::WHITE,
        }
    }

    pub fn clear(&mut self) {
        self.color_buffer = Image::gen_image_color(self.width as i32, self.height as i32, self.background_color);
    }

    pub fn set_pixel(&mut self, x: u32, y: u32) {
        if x < self.width && y < self.height {
            self.color_buffer.draw_pixel(x as i32, y as i32, self.current_color);
        }
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn _render_to_file(&self, file_path: &str) {
        self.color_buffer.export_image(file_path);
    }

    // Draw framebuffer to screen and optionally overlay FPS as text
    pub fn swap_buffers(
        &self,
        window: &mut RaylibHandle,
        raylib_thread: &RaylibThread,
        fps: Option<i32>,
    ) {
        if let Ok(texture) = window.load_texture_from_image(raylib_thread, &self.color_buffer) {
            // Preserve aspect ratio: compute destination rect that fits the window without stretching
            let screen_w = window.get_screen_width();
            let screen_h = window.get_screen_height();

            let mut renderer = window.begin_drawing(raylib_thread);
            let fb_w = self.width as f32;
            let fb_h = self.height as f32;
            let screen_aspect = screen_w as f32 / screen_h as f32;
            let fb_aspect = fb_w / fb_h;

            let (dest_w, dest_h) = if fb_aspect > screen_aspect {
                // framebuffer is wider relative to screen -> fit by width
                (screen_w as f32, screen_w as f32 / fb_aspect)
            } else {
                // fit by height
                (screen_h as f32 * fb_aspect, screen_h as f32)
            };

            let dest_x = ((screen_w as f32 - dest_w) / 2.0) as i32;
            let dest_y = ((screen_h as f32 - dest_h) / 2.0) as i32;

            // source rectangle covers whole texture
            let src = Rectangle::new(0.0, 0.0, fb_w, fb_h);
            // dest rectangle where to draw the texture
            let dest = Rectangle::new(dest_x as f32, dest_y as f32, dest_w, dest_h);
            // origin for rotation/scaling
            let origin = Vector2::new(0.0, 0.0);

            renderer.draw_texture_pro(&texture, src, dest, origin, 0.0, Color::WHITE);
            if let Some(f) = fps {
                let txt = format!("FPS: {}", f);
                // draw semi-transparent background for readability
                renderer.draw_rectangle(10, 10, 90, 26, Color::new(0, 0, 0, 120));
                renderer.draw_text(&txt, 16, 14, 20, Color::RAYWHITE);
            }
        }
    }

    // Draw framebuffer and overlay with coin counter
    pub fn swap_buffers_with_coins(
        &self,
        window: &mut RaylibHandle,
        raylib_thread: &RaylibThread,
        fps: Option<i32>,
        coins_collected: usize,
        total_coins: usize,
        current_level: i32,
    ) {
        if let Ok(texture) = window.load_texture_from_image(raylib_thread, &self.color_buffer) {
            // Preserve aspect ratio: compute destination rect that fits the window without stretching
            let screen_w = window.get_screen_width();
            let screen_h = window.get_screen_height();
            let mut renderer = window.begin_drawing(raylib_thread);
            let fb_w = self.width as f32;
            let fb_h = self.height as f32;
            let screen_aspect = screen_w as f32 / screen_h as f32;
            let fb_aspect = fb_w / fb_h;

            let (dest_w, dest_h) = if fb_aspect > screen_aspect {
                // framebuffer is wider relative to screen -> fit by width
                (screen_w as f32, screen_w as f32 / fb_aspect)
            } else {
                // fit by height
                (screen_h as f32 * fb_aspect, screen_h as f32)
            };

            let dest_x = ((screen_w as f32 - dest_w) / 2.0) as i32;
            let dest_y = ((screen_h as f32 - dest_h) / 2.0) as i32;

            // source rectangle covers whole texture
            let src = Rectangle::new(0.0, 0.0, fb_w, fb_h);
            // dest rectangle where to draw the texture
            let dest = Rectangle::new(dest_x as f32, dest_y as f32, dest_w, dest_h);
            // origin for rotation/scaling
            let origin = Vector2::new(0.0, 0.0);

            renderer.draw_texture_pro(&texture, src, dest, origin, 0.0, Color::WHITE);
            
            if let Some(f) = fps {
                let txt = format!("FPS: {}", f);
                // draw semi-transparent background for readability
                renderer.draw_rectangle(10, 10, 90, 26, Color::new(0, 0, 0, 120));
                renderer.draw_text(&txt, 16, 14, 20, Color::RAYWHITE);
            }
            
            // Draw coin counter
            let coins_text = format!("Monedas: {}/{}", coins_collected, total_coins);
            renderer.draw_rectangle(screen_w - 210, 10, 200, 30, Color::new(0, 0, 0, 120));
            renderer.draw_text(&coins_text, screen_w - 200, 20, 24, Color::GOLD);
            
            // Draw level indicator
            let level_text = format!("Nivel: {}", current_level);
            renderer.draw_rectangle(screen_w / 2 - 50, 10, 100, 30, Color::new(0, 0, 0, 120));
            renderer.draw_text(&level_text, screen_w / 2 - 40, 20, 24, Color::CYAN);
        }
    }
}
