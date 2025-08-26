// Simple animation helpers for UI/menu
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
