use ropey::Rope;

pub struct Editor {
    buffer: Rope,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: Rope::new(),
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.buffer = Rope::from_str(text);
    }

    pub fn get_text(&self) -> String {
        self.buffer.to_string()
    }
}
