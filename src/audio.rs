use std::ffi::CString;
use std::path::Path;

pub struct AudioManager {
    initialized: bool,
    music: Option<raylib::ffi::Music>,
}

impl AudioManager {
    pub fn new() -> Self {
        Self { initialized: false, music: None }
    }

    pub fn init(&mut self) {
        if !self.initialized {
            unsafe { raylib::ffi::InitAudioDevice(); }
            self.initialized = true;
        }
    }

    fn find_oggs() -> Vec<String> {
        let mut oggs = Vec::new();
        if let Ok(entries) = std::fs::read_dir("sounds") {
            for e in entries.flatten() {
                if let Some(name) = e.path().file_name().and_then(|n| n.to_str()) {
                    if name.to_lowercase().ends_with(".ogg") {
                        oggs.push(format!("sounds/{}", name));
                    }
                }
            }
            oggs.sort();
        }
        if Path::new("music.ogg").exists() {
            oggs.push("music.ogg".to_string());
        }
        oggs
    }

    fn load_and_play_internal(path: &str) -> Option<raylib::ffi::Music> {
        unsafe {
            if let Ok(cpath) = CString::new(path.to_string()) {
                let m = raylib::ffi::LoadMusicStream(cpath.as_ptr());
                if raylib::ffi::IsMusicValid(m) {
                    raylib::ffi::PlayMusicStream(m);
                    eprintln!("[info] playing music: {}", path);
                    return Some(m);
                } else {
                    eprintln!("[warn] failed to load music: {}", path);
                }
            } else {
                eprintln!("[warn] invalid music path: {}", path);
            }
        }
        None
    }

    pub fn play_menu_track(&mut self) {
        // NOTE: swapped: menu should play the gameplay track (sounds/game.ogg) per user request
        let oggs = Self::find_oggs();
        if Path::new("sounds/game.ogg").exists() {
            if let Some(m) = Self::load_and_play_internal("sounds/game.ogg") {
                self.music = Some(m);
                return;
            }
        }
        // fallback: if there are any oggs, play the first one
        if !oggs.is_empty() {
            if let Some(m) = Self::load_and_play_internal(&oggs[0]) {
                self.music = Some(m);
            }
        }
    }

    pub fn play_game_track(&mut self) {
        // NOTE: swapped: gameplay should play the menu track (sounds/menu.ogg) per user request
        let oggs = Self::find_oggs();
        if Path::new("sounds/menu.ogg").exists() {
            if let Some(m) = Self::load_and_play_internal("sounds/menu.ogg") {
                self.music = Some(m);
                return;
            }
        }
        // prefer second file if available, else first
        if oggs.len() >= 2 {
            if let Some(m) = Self::load_and_play_internal(&oggs[1]) {
                self.music = Some(m);
                return;
            }
        }
        if oggs.len() == 1 {
            if let Some(m) = Self::load_and_play_internal(&oggs[0]) {
                self.music = Some(m);
            }
        }
    }

    pub fn stop_unload(&mut self) {
        if let Some(m) = self.music.take() {
            unsafe {
                raylib::ffi::StopMusicStream(m);
                raylib::ffi::UnloadMusicStream(m);
            }
        }
    }

    pub fn update(&self) {
        if let Some(m) = self.music {
            unsafe { raylib::ffi::UpdateMusicStream(m); }
        }
    }

    pub fn cleanup(&mut self) {
        self.stop_unload();
        if self.initialized {
            unsafe { raylib::ffi::CloseAudioDevice(); }
            self.initialized = false;
        }
    }
}
