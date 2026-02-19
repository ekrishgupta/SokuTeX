use ropey::Rope;

pub struct Editor {
    pub buffer: Rope,
    pub cursor: usize,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: Rope::new(),
            cursor: 0,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.buffer.insert_char(self.cursor, c);
        self.cursor += 1;
    }

    pub fn delete_back(&mut self) {
        if self.cursor > 0 {
            self.buffer.remove(self.cursor - 1..self.cursor);
            self.cursor -= 1;
        }
    }

    pub fn get_text(&self) -> String {
        self.buffer.to_string()
    }
}
