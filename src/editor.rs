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
    pub history: Vec<(Rope, usize, EditorMode, Option<usize>)>,
    pub redo_stack: Vec<(Rope, usize, EditorMode, Option<usize>)>,
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
            history: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn snapshot(&mut self) {
        self.history.push((self.buffer.clone(), self.cursor, self.mode, self.visual_anchor));
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some((buf, cur, mode, anchor)) = self.history.pop() {
            self.redo_stack.push((self.buffer.clone(), self.cursor, self.mode, self.visual_anchor));
            self.buffer = buf;
            self.cursor = cur;
            self.mode = mode;
            self.visual_anchor = anchor;
        }
    }

    pub fn redo(&mut self) {
        if let Some((buf, cur, mode, anchor)) = self.redo_stack.pop() {
            self.history.push((self.buffer.clone(), self.cursor, self.mode, self.visual_anchor));
            self.buffer = buf;
            self.cursor = cur;
            self.mode = mode;
            self.visual_anchor = anchor;
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

    fn handle_visual_key(&mut self, c: char) {
        match c {
            '\u{1b}' => {
                // Escape
                self.mode = EditorMode::Normal;
                self.visual_anchor = None;
            }
            'h' => self.move_left(),
            'j' => self.move_down(),
            'k' => self.move_up(),
            'l' => self.move_right(),
            'd' | 'x' => {
                self.delete_selection();
                self.mode = EditorMode::Normal;
                self.visual_anchor = None;
            }
            _ => {}
        }
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

    fn delete_selection(&mut self) {
        if let Some(anchor) = self.visual_anchor {
            let start = anchor.min(self.cursor);
            let end = anchor.max(self.cursor);
            if start < end {
                self.buffer.remove(start..end);
                self.cursor = start;
            }
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

    fn move_to_line_start(&mut self) {
        let line_idx = self.buffer.char_to_line(self.cursor);
        self.cursor = self.buffer.line_to_char(line_idx);
    }

    fn move_to_line_end(&mut self) {
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

    pub fn move_to_line(&mut self, line_num: usize) {
        if line_num > 0 && line_num <= self.buffer.len_lines() {
            self.cursor = self.buffer.line_to_char(line_num - 1);
        }
    }

    pub fn expand_or_jump_snippet(&mut self, autocomplete: &crate::autocomplete::AutocompleteEngine) -> Option<(usize, usize)> {
        let mut start_idx = self.cursor;
        while start_idx > 0 {
            let c = self.buffer.char(start_idx - 1);
            if c.is_whitespace() {
                break;
            }
            start_idx -= 1;
        }

        if start_idx < self.cursor {
            let word: String = self.buffer.slice(start_idx..self.cursor).chars().collect();
            if let Some(snippet) = autocomplete.get_snippet(&word) {
                // Remove word
                self.buffer.remove(start_idx..self.cursor);
                self.cursor = start_idx;

                // Insert snippet
                let snippet_start = self.cursor;
                for c in snippet.chars() {
                    self.buffer.insert_char(self.cursor, c);
                    self.cursor += 1;
                }
                
                // Reset cursor to start of snippet so jump logic finds the first placeholder
                self.cursor = snippet_start;
            }
        }

        let len = self.buffer.len_chars();
        let mut search_idx = self.cursor;
        
        // Find next `$<digit>`
        while search_idx < len.saturating_sub(1) {
            if self.buffer.char(search_idx) == '$' {
                let next_char = self.buffer.char(search_idx + 1);
                if next_char.is_ascii_digit() {
                    let mut end_idx = search_idx + 2;
                    while end_idx < len && self.buffer.char(end_idx).is_ascii_digit() {
                        end_idx += 1;
                    }
                    self.cursor = end_idx;
                    return Some((search_idx, end_idx));
                }
            }
            search_idx += 1;
        }

        // Optional wrap around, or just return None
        let mut search_idx = 0;
        while search_idx < self.cursor.saturating_sub(1) {
            if self.buffer.char(search_idx) == '$' {
                let next_char = self.buffer.char(search_idx + 1);
                if next_char.is_ascii_digit() {
                    let mut end_idx = search_idx + 2;
                    while end_idx < len && self.buffer.char(end_idx).is_ascii_digit() {
                        end_idx += 1;
                    }
                    self.cursor = end_idx;
                    return Some((search_idx, end_idx));
                }
            }
            search_idx += 1;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vim_modes() {
        let mut editor = Editor::new();
        assert_eq!(editor.mode, EditorMode::Normal);
        
        // Enter insert mode
        editor.handle_key('i');
        assert_eq!(editor.mode, EditorMode::Insert);
        
        // Type some text
        editor.handle_key('h');
        editor.handle_key('e');
        editor.handle_key('l');
        editor.handle_key('l');
        editor.handle_key('o');
        assert_eq!(editor.get_text(), "hello");
        
        // Escape to normal mode
        editor.handle_key('\u{1b}');
        assert_eq!(editor.mode, EditorMode::Normal);
        
        // Move left and delete char (vim 'x' at cursor)
        editor.handle_key('h'); // cursor at 'o'
        editor.handle_key('h'); // cursor at 'l'
        editor.handle_key('x'); // delete 'l'
        assert_eq!(editor.get_text(), "helo");
    }
}
