use ropey::Rope;

pub struct SyncTex;

impl SyncTex {
    /// Maps a PDF location (simulated here) to an editor character offset
    pub fn pdf_to_editor(rope: &Rope, line: usize, col: usize) -> usize {
        if line >= rope.len_lines() {
            return rope.len_chars();
        }
        let line_start = rope.line_to_char(line);
        let line_len = rope.line(line).len_chars();
        line_start + col.min(line_len)
    }

    /// Maps an editor character offset to a PDF location (line, col)
    pub fn editor_to_pdf(rope: &Rope, offset: usize) -> (usize, usize) {
        let line = rope.char_to_line(offset.min(rope.len_chars()));
        let line_start = rope.line_to_char(line);
        let col = offset - line_start;
        (line, col)
    }
}
