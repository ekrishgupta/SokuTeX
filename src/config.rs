#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompileBackend {
    Internal,
    Shadow,
    #[allow(dead_code)]
    Tectonic,
    Latexmk,
}

pub struct Config {
    pub background_color: [f32; 4],
}

impl Default for Config {
    fn default() -> Self {
        Self {
            background_color: [0.05, 0.05, 0.05, 1.0], // Minimalist dark
        }
    }
}
