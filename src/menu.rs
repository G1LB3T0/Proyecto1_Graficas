use crate::framebuffer::Framebuffer;
use crate::textures::TextureAtlas;
use raylib::prelude::*;

pub enum MenuAction {
    StartLevel(i32),
    Quit,
}

enum MenuState {
    Main,
    LevelSelect,
}

pub fn run_menu(window: &mut RaylibHandle, thread: &RaylibThread, framebuffer: &mut Framebuffer, textures: &TextureAtlas, audio: &mut crate::audio::AudioManager) -> MenuAction {
    let mut menu_state = MenuState::Main;
    let mut main_selection: usize = 0; // 0 = Jugar, 1 = Salir
    let mut level_selection: i32 = 1; // 1, 2, 3

    loop {
        // Check if window should close
        if window.window_should_close() {
            return MenuAction::Quit;
        }
        
        framebuffer.clear();

        // Draw background (same as before)
        let fb_w = framebuffer.width as u32;
        let fb_h = framebuffer.height as u32;

        // Determine if we have a menu image and its native size
        let menu_dims = textures.menu.as_ref().map(|m| (m.w, m.h));

        if let Some((mw, mh)) = menu_dims {
            // compute scale that fits menu inside framebuffer without stretching
            let scale = (fb_w as f32 / mw as f32).min(fb_h as f32 / mh as f32).max(1e-6);
            let tw = (mw as f32 * scale).floor() as u32;
            let th = (mh as f32 * scale).floor() as u32;
            let ox = ((fb_w - tw) / 2) as isize;
            let oy = ((fb_h - th) / 2) as isize;

            // draw background dark
            let bg = Color::new(8,8,16,255);
            for y in 0..fb_h {
                for x in 0..fb_w {
                    framebuffer.set_current_color(bg);
                    framebuffer.set_pixel(x, y);
                }
            }

            // sample menu texture only into centered rect to preserve aspect (animated)
            for y in 0..th {
                for x in 0..tw {
                    let u = x as f32 / (tw as f32 - 1.0).max(1.0);
                    let v = y as f32 / (th as f32 - 1.0).max(1.0);
                    let col = textures.sample_menu(u, v);
                    let px = ox + x as isize;
                    let py = oy + y as isize;
                    if px >= 0 && py >= 0 {
                        let pxu = px as u32;
                        let pyu = py as u32;
                        if pxu < fb_w && pyu < fb_h {
                            framebuffer.set_current_color(col);
                            framebuffer.set_pixel(pxu, pyu);
                        }
                    }
                }
            }
        } else {
            // no menu texture - fallback to full-screen sampling
            for y in 0..fb_h {
                for x in 0..fb_w {
                    let u = x as f32 / fb_w as f32;
                    let v = y as f32 / fb_h as f32;
                    let col = textures.sample_menu(u, v);
                    framebuffer.set_current_color(col);
                    framebuffer.set_pixel(x, y);
                }
            }
        }

        // Input handling based on current menu state
        match menu_state {
            MenuState::Main => {
                if window.is_key_pressed(KeyboardKey::KEY_DOWN) || window.is_key_pressed(KeyboardKey::KEY_S) {
                    main_selection = (main_selection + 1) % 2;
                }
                if window.is_key_pressed(KeyboardKey::KEY_UP) || window.is_key_pressed(KeyboardKey::KEY_W) {
                    main_selection = (main_selection + 2 - 1) % 2;
                }
                if window.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    if main_selection == 0 {
                        menu_state = MenuState::LevelSelect;
                    } else {
                        return MenuAction::Quit;
                    }
                }
                if window.is_key_pressed(KeyboardKey::KEY_Q) {
                    return MenuAction::Quit;
                }
            }
            MenuState::LevelSelect => {
                if window.is_key_pressed(KeyboardKey::KEY_DOWN) || window.is_key_pressed(KeyboardKey::KEY_S) {
                    level_selection = if level_selection < 3 { level_selection + 1 } else { 1 };
                }
                if window.is_key_pressed(KeyboardKey::KEY_UP) || window.is_key_pressed(KeyboardKey::KEY_W) {
                    level_selection = if level_selection > 1 { level_selection - 1 } else { 3 };
                }
                if window.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    return MenuAction::StartLevel(level_selection);
                }
                if window.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    menu_state = MenuState::Main;
                }
                if window.is_key_pressed(KeyboardKey::KEY_Q) {
                    return MenuAction::Quit;
                }
            }
        }

        // Draw overlay text via raylib
        let screen_w = window.get_screen_width();
        let screen_h = window.get_screen_height();
        if let Ok(texture) = window.load_texture_from_image(thread, &framebuffer.color_buffer) {
            let mut d = window.begin_drawing(thread);
            let src = Rectangle::new(0.0, 0.0, framebuffer.width as f32, framebuffer.height as f32);
            let dest = Rectangle::new(0.0, 0.0, screen_w as f32, screen_h as f32);
            let origin = Vector2::new(0.0,0.0);
            d.draw_texture_pro(&texture, src, dest, origin, 0.0, Color::WHITE);

            let cx = screen_w / 2;

            match menu_state {
                MenuState::Main => {
                    // Draw main menu
                    let opt_y = screen_h / 2 - 50;
                    let play_color = if main_selection == 0 { Color::YELLOW } else { Color::WHITE };
                    let quit_color = if main_selection == 1 { Color::YELLOW } else { Color::WHITE };
                    d.draw_text("JUGAR", cx - 40, opt_y, 40, play_color);
                    d.draw_text("SALIR", cx - 40, opt_y + 60, 40, quit_color);
                }
                MenuState::LevelSelect => {
                    // Draw level selection
                    let title_y = screen_h / 2 - 200;
                    d.draw_text("SELECCIONAR NIVEL", cx - 150, title_y, 40, Color::WHITE);

                    let level_y = screen_h / 2 - 80;
                    
                    let level1_color = if level_selection == 1 { Color::YELLOW } else { Color::WHITE };
                    let level2_color = if level_selection == 2 { Color::YELLOW } else { Color::WHITE };
                    let level3_color = if level_selection == 3 { Color::YELLOW } else { Color::WHITE };

                    d.draw_text("NIVEL 1 - FACIL (2 fichas)", cx - 140, level_y, 30, level1_color);
                    d.draw_text("NIVEL 2 - MEDIO (4 fichas)", cx - 150, level_y + 60, 30, level2_color);
                    d.draw_text("NIVEL 3 - DIFICIL (6 fichas)", cx - 160, level_y + 120, 30, level3_color);

                    // Instructions
                    d.draw_text("ESC = Volver | ENTER = Jugar", cx - 140, level_y + 200, 20, Color::GRAY);
                }
            }
        }

        // update audio streaming buffers for menu music
        audio.update();
        // small sleep to avoid busy loop
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
