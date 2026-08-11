const FULLWIDTH_RANGES: &[(u32, u32)] = &[
    (0x1100, 0x115F),
    (0x2E80, 0x303E),
    (0x3041, 0x33FF),
    (0x3400, 0x4DBF),
    (0x4E00, 0x9FFF),
    (0xA000, 0xA4CF),
    (0xAC00, 0xD7A3),
    (0xF900, 0xFAFF),
    (0xFE30, 0xFE4F),
    (0xFF00, 0xFF60),
    (0xFFE0, 0xFFE6),
    (0x1F300, 0x1F64F),
    (0x1F900, 0x1F9FF),
    (0x20000, 0x3FFFD),
];

pub fn char_width_kind(cp: u32) -> u8 {
    for (start, end) in FULLWIDTH_RANGES {
        if cp >= *start && cp <= *end {
            return 2;
        }
    }
    1
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorBuffer {
    text: String,
    cursor_line: u32,
    cursor_column: u32,
}

impl EditorBuffer {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            cursor_line: 0,
            cursor_column: 0,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor_line(&self) -> u32 {
        self.cursor_line
    }

    pub fn cursor_column(&self) -> u32 {
        self.cursor_column
    }

    pub fn line_count(&self) -> usize {
        self.text.split('\n').count()
    }

    pub fn line_text(&self, line: usize) -> &str {
        self.text.split('\n').nth(line).unwrap_or("")
    }

    pub fn line_width_columns(&self, line: usize) -> u32 {
        self.line_text(line).chars().map(|c| char_width_kind(c as u32) as u32).sum()
    }

    pub fn line_start_index(&self, line: usize) -> usize {
        let mut current = 0;
        for l in 0..line {
            current += self.line_text(l).len() + 1;
        }
        current.min(self.text.len())
    }

    pub fn column_to_char_index(&self, line: usize, column: u32) -> usize {
        let line_text = self.line_text(line);
        self.line_start_index(line) + column_to_char_index_impl(&line_text, column)
    }

    pub fn insert_at_cursor(&mut self, insert: &str) {
        let index = self.column_to_char_index(self.cursor_line as usize, self.cursor_column);
        self.text.insert_str(index, insert);
        for c in insert.chars() {
            if c == '\n' {
                self.cursor_line += 1;
                self.cursor_column = 0;
            } else {
                self.cursor_column += char_width_kind(c as u32) as u32;
            }
        }
    }

    pub fn newline_at_cursor(&mut self) {
        self.insert_at_cursor("\n");
    }

    pub fn backspace_at_cursor(&mut self) {
        if self.cursor_column == 0 {
            if self.cursor_line > 0 {
                let prev_line = self.cursor_line as usize - 1;
                self.cursor_line -= 1;
                self.cursor_column = self.line_width_columns(prev_line);
                let index = self.line_start_index(self.cursor_line as usize) + self.line_text(self.cursor_line as usize).len();
                self.text.remove(index);
            }
            return;
        }
        let (index, width) = self.char_before_cursor();
        self.text.remove(index);
        self.cursor_column = self.cursor_column.saturating_sub(width);
    }

    pub fn delete_at_cursor(&mut self) {
        let index = self.column_to_char_index(self.cursor_line as usize, self.cursor_column);
        if index >= self.text.len() {
            return;
        }
        let c = self.text[index..].chars().next().unwrap();
        self.text.remove(index);
        if c == '\n' {
            self.cursor_column = 0;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_column == 0 {
            if self.cursor_line > 0 {
                self.cursor_line -= 1;
                self.cursor_column = self.line_width_columns(self.cursor_line as usize);
            }
            return;
        }
        let (_, width) = self.char_before_cursor();
        self.cursor_column -= width;
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_column >= self.line_width_columns(self.cursor_line as usize) {
            if self.cursor_line as usize + 1 < self.line_count() {
                self.cursor_line += 1;
                self.cursor_column = 0;
            }
            return;
        }
        let width = self.char_at_cursor();
        self.cursor_column += width;
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_line == 0 {
            self.cursor_column = 0;
            return;
        }
        self.cursor_line -= 1;
        let width = self.line_width_columns(self.cursor_line as usize);
        if self.cursor_column > width {
            self.cursor_column = width;
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor_line as usize + 1 >= self.line_count() {
            self.cursor_column = self.line_width_columns(self.cursor_line as usize);
            return;
        }
        self.cursor_line += 1;
        let width = self.line_width_columns(self.cursor_line as usize);
        if self.cursor_column > width {
            self.cursor_column = width;
        }
    }

    pub fn move_cursor_to_line_start(&mut self) {
        self.cursor_column = 0;
    }

    pub fn move_cursor_to_line_end(&mut self) {
        self.cursor_column = self.line_width_columns(self.cursor_line as usize);
    }

    fn char_at_cursor(&self) -> u32 {
        let index = self.column_to_char_index(self.cursor_line as usize, self.cursor_column);
        self.text[index..].chars().next().map(|c| char_width_kind(c as u32) as u32).unwrap_or(1)
    }

    fn char_before_cursor(&self) -> (usize, u32) {
        let line_text = self.line_text(self.cursor_line as usize);
        let mut width: u32 = 0;
        let mut last: Option<(u32, u32)> = None;
        for c in line_text.chars() {
            if width >= self.cursor_column {
                break;
            }
            let w = char_width_kind(c as u32) as u32;
            last = Some((width, w));
            width += w;
        }
        let (start_column, width) = last.unwrap_or((0, 1));
        let index = self.line_start_index(self.cursor_line as usize) + column_to_char_index_impl(&line_text, start_column);
        (index, width)
    }
}

fn column_to_char_index_impl(line_text: &str, column: u32) -> usize {
    let mut width = 0;
    let mut index = 0;
    for (i, c) in line_text.char_indices() {
        if width >= column {
            index = i;
            break;
        }
        width += char_width_kind(c as u32) as u32;
        index = i + c.len_utf8();
    }
    if width < column {
        index = line_text.len();
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_text_advances_cursor() {
        let mut buffer = EditorBuffer::new("");
        buffer.insert_at_cursor("ab");
        assert_eq!(buffer.text(), "ab");
        assert_eq!(buffer.cursor_column(), 2);
    }

    #[test]
    fn insert_in_middle_of_line() {
        let mut buffer = EditorBuffer::new("abcd");
        buffer.cursor_column = 2;
        buffer.insert_at_cursor("X");
        assert_eq!(buffer.text(), "abXcd");
        assert_eq!(buffer.cursor_column(), 3);
    }

    #[test]
    fn backspace_removes_previous_char() {
        let mut buffer = EditorBuffer::new("ab");
        buffer.cursor_column = 2;
        buffer.backspace_at_cursor();
        assert_eq!(buffer.text(), "a");
        assert_eq!(buffer.cursor_column(), 1);
    }

    #[test]
    fn backspace_at_line_start_joins_lines() {
        let mut buffer = EditorBuffer::new("abc\ndef");
        buffer.cursor_line = 1;
        buffer.backspace_at_cursor();
        assert_eq!(buffer.text(), "abcdef");
        assert_eq!(buffer.cursor_line(), 0);
        assert_eq!(buffer.cursor_column(), 3);
    }

    #[test]
    fn newline_splits_line() {
        let mut buffer = EditorBuffer::new("abc");
        buffer.cursor_column = 1;
        buffer.newline_at_cursor();
        assert_eq!(buffer.text(), "a\nbc");
        assert_eq!(buffer.cursor_line(), 1);
        assert_eq!(buffer.cursor_column(), 0);
    }

    #[test]
    fn delete_removes_char_at_cursor() {
        let mut buffer = EditorBuffer::new("abc");
        buffer.cursor_column = 1;
        buffer.delete_at_cursor();
        assert_eq!(buffer.text(), "ac");
    }

    #[test]
    fn move_cursor_left_right() {
        let mut buffer = EditorBuffer::new("abc\ndef");
        buffer.cursor_line = 1;
        buffer.cursor_column = 3;
        buffer.move_cursor_right();
        assert_eq!(buffer.cursor_line(), 1);
        assert_eq!(buffer.cursor_column(), 3);
        buffer.move_cursor_left();
        assert_eq!(buffer.cursor_column(), 2);
        buffer.cursor_column = 0;
        buffer.move_cursor_left();
        assert_eq!(buffer.cursor_line(), 0);
        assert_eq!(buffer.cursor_column(), 3);
    }

    #[test]
    fn move_cursor_up_down_clamps() {
        let mut buffer = EditorBuffer::new("abc\nd");
        buffer.cursor_line = 1;
        buffer.cursor_column = 2;
        buffer.move_cursor_up();
        assert_eq!(buffer.cursor_line(), 0);
        assert_eq!(buffer.cursor_column(), 2);
        buffer.move_cursor_down();
        assert_eq!(buffer.cursor_line(), 1);
        assert_eq!(buffer.cursor_column(), 1);
    }

    #[test]
    fn cjk_double_width_columns() {
        let mut buffer = EditorBuffer::new("");
        buffer.insert_at_cursor("你");
        assert_eq!(buffer.cursor_column(), 2);
        buffer.insert_at_cursor("a");
        assert_eq!(buffer.cursor_column(), 3);
        assert_eq!(buffer.line_width_columns(0), 3);
    }

    #[test]
    fn cjk_column_to_char_index_roundtrip() {
        let mut buffer = EditorBuffer::new("你好world");
        buffer.cursor_column = 4;
        buffer.insert_at_cursor("X");
        assert_eq!(buffer.text(), "你好Xworld");
        assert_eq!(buffer.cursor_column(), 5);
        assert_eq!(buffer.column_to_char_index(0, 4), "你好".len());
    }

    #[test]
    fn cjk_backspace_moves_by_two_columns() {
        let mut buffer = EditorBuffer::new("a你b");
        buffer.cursor_column = 3;
        buffer.backspace_at_cursor();
        assert_eq!(buffer.text(), "ab");
        assert_eq!(buffer.cursor_column(), 1);
    }

    #[test]
    fn cjk_caret_moves_in_double_width_steps() {
        let mut buffer = EditorBuffer::new("你a");
        buffer.move_cursor_right();
        assert_eq!(buffer.cursor_column(), 2);
        buffer.move_cursor_right();
        assert_eq!(buffer.cursor_column(), 3);
        buffer.move_cursor_left();
        assert_eq!(buffer.cursor_column(), 2);
    }
}
