use ropey::Rope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Normal,
    Insert,
    Visual,
}

#[allow(dead_code)]
pub struct Editor {
    pub buffer: Rope,
    pub cursor: usize,
    pub mode: EditorMode,
    pub entries: Vec<crate::bib::BibEntry>,
    pub visual_anchor: Option<usize>,
}

#[allow(dead_code)]
impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: Rope::new(),
            cursor: 0,
            mode: EditorMode::Normal,
            entries: Vec::new(),
            visual_anchor: None,
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

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.buffer.len_chars() {
            self.cursor += 1;
        }
    }

    pub fn move_up(&mut self) {
        let line_idx = self.buffer.char_to_line(self.cursor);
        if line_idx > 0 {
            let col = self.cursor - self.buffer.line_to_char(line_idx);
            let prev_line_start = self.buffer.line_to_char(line_idx - 1);
            let prev_line_len = self.buffer.line(line_idx - 1).len_chars();
            // Handle trailing newline
            let max_col = if prev_line_len > 0 && self.buffer.char(prev_line_start + prev_line_len - 1) == '\n' {
                prev_line_len - 1
            } else {
                prev_line_len
            };
            self.cursor = prev_line_start + col.min(max_col);
        }
    }

    pub fn move_down(&mut self) {
        let line_idx = self.buffer.char_to_line(self.cursor);
        if line_idx < self.buffer.len_lines() - 1 {
            let col = self.cursor - self.buffer.line_to_char(line_idx);
            let next_line_start = self.buffer.line_to_char(line_idx + 1);
            let next_line_len = self.buffer.line(line_idx + 1).len_chars();
            let max_col = if next_line_len > 0 && self.buffer.char(next_line_start + next_line_len - 1) == '\n' {
                next_line_len - 1
            } else {
                next_line_len
            };
            self.cursor = next_line_start + col.min(max_col);
        }
    }

    pub fn move_to_line_start(&mut self) {
        let line_idx = self.buffer.char_to_line(self.cursor);
        self.cursor = self.buffer.line_to_char(line_idx);
    }

    pub fn move_to_line_end(&mut self) {
        let line_idx = self.buffer.char_to_line(self.cursor);
        let line_start = self.buffer.line_to_char(line_idx);
        let line_len = self.buffer.line(line_idx).len_chars();
        if line_len > 0 && self.buffer.char(line_start + line_len - 1) == '\n' {
            self.cursor = line_start + line_len - 1;
        } else {
            self.cursor = line_start + line_len;
        }
    }

    pub fn get_text(&self) -> String {
        self.buffer.to_string()
    }
}
