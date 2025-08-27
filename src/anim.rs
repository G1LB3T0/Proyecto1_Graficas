// Simple animation helpers for UI/menu and game objects
pub struct MenuAnimation {
    t: f32,
}

impl MenuAnimation {
    pub fn new() -> Self {
        Self { t: 0.0 }
    }

    // advance animation by dt seconds
    pub fn update(&mut self, dt: f32) {
        self.t += dt;
        // keep t bounded to avoid float runaway
        if self.t > std::f32::consts::TAU * 100.0 {
            self.t = self.t % (std::f32::consts::TAU);
        }
    }

    // small pulsing scale around 1.0 (±0.03)
    pub fn scale(&self) -> f32 {
        1.0 + 0.03 * (self.t * 1.5).sin()
    }

    // small vertical bob (pixels)
    pub fn bob(&self) -> f32 {
        6.0 * (self.t * 0.7).sin()
    }
}

// Coin animation helpers
pub struct CoinAnimation;

impl CoinAnimation {
    // Calculate the current frame for sprite animation (12 frames total)
    pub fn get_current_frame(animation_time: f32) -> usize {
        let num_frames = 12;
        let frame_time = (2.0 * std::f32::consts::PI) / num_frames as f32;
        ((animation_time / frame_time) as usize) % num_frames
    }

    // Calculate floating motion offset in pixels
    pub fn get_float_offset(animation_time: f32) -> f32 {
        8.0 * (animation_time * 0.8).sin()
    }

    // Update coin animation time with proper wrapping
    pub fn update_time(current_time: f32, delta: f32) -> f32 {
        let new_time = current_time + delta;
        if new_time > std::f32::consts::TAU {
            new_time % std::f32::consts::TAU
        } else {
            new_time
        }
    }

    // Get frame offset for spritesheet sampling
    pub fn get_frame_offset(animation_time: f32, frame_width: u32) -> u32 {
        let current_frame = Self::get_current_frame(animation_time);
        current_frame as u32 * frame_width
    }
}
