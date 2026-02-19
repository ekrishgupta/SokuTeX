#[allow(dead_code)]
pub struct Config {
    pub background_color: [f32; 4],
    pub font_size: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            background_color: [0.05, 0.05, 0.05, 1.0], // Minimalist dark
            font_size: 14.0,
        }
    }
}
