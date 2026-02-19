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

    pub fn handle_key(&mut self, c: char) {
        match self.mode {
            EditorMode::Normal => self.handle_normal_key(c),
            EditorMode::Insert => {
                if c == '\u{1b}' {
                    // Escape
                    self.mode = EditorMode::Normal;
                } else {
                    self.insert_char(c);
                }
            }
            EditorMode::Visual => self.handle_visual_key(c),
        }
    }

    fn handle_normal_key(&mut self, c: char) {
        match c {
            'i' => self.mode = EditorMode::Insert,
            'v' => {
                self.mode = EditorMode::Visual;
                self.visual_anchor = Some(self.cursor);
            }
            'h' => self.move_left(),
            'j' => self.move_down(),
            'k' => self.move_up(),
            'l' => self.move_right(),
            'x' => self.delete_char(),
            'a' => {
                self.move_right();
                self.mode = EditorMode::Insert;
            }
            '0' => self.move_to_line_start(),
            '$' => self.move_to_line_end(),
            _ => {}
        }
    }

    fn handle_visual_key(&mut self, _c: char) {
        // To be implemented in next commit
    }

    pub fn insert_char(&mut self, c: char) {
        self.buffer.insert_char(self.cursor, c);
        self.cursor += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor < self.buffer.len_chars() {
            self.buffer.remove(self.cursor..self.cursor + 1);
        }
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
