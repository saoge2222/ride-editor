use crate::render::render_font::Font;
use crate::render::render_shape_text::{char_width_kind, TextShaper};
use crate::widgets::widgets_definition::WidgetId;

pub struct EditorBuffer {
    pub id: WidgetId,
    pub path: String,
    pub text: String,
    pub cursor_line: u32,
    pub cursor_column: u32,
}

impl EditorBuffer {
    pub fn new(id: WidgetId, path: String, text: String) -> Self {
        Self {
            id,
            path,
            text,
            cursor_line: 0,
            cursor_column: 0,
        }
    }

    pub fn line_count(&self) -> usize {
        self.text.split('\n').count()
    }

    pub fn line_text(&self, line: usize) -> &str {
        self.text.split('\n').nth(line).unwrap_or("")
    }

    pub fn caret_x(
        &self,
        shaper: &TextShaper,
        primary_font: &Font,
        cjk_font: Option<&Font>,
        cell_width: f32,
    ) -> f32 {
        let line = self.line_text(self.cursor_line as usize);
        let shaped = shaper.shape(line, primary_font, cjk_font, cell_width);
        shaper.column_to_x(&shaped, self.cursor_column as usize)
    }

    pub fn column_at_x(
        &self,
        shaper: &TextShaper,
        primary_font: &Font,
        cjk_font: Option<&Font>,
        cell_width: f32,
        x: f32,
    ) -> usize {
        let line = self.line_text(self.cursor_line as usize);
        let shaped = shaper.shape(line, primary_font, cjk_font, cell_width);
        shaper.x_to_column(&shaped, x)
    }

    pub fn line_width(
        &self,
        shaper: &TextShaper,
        primary_font: &Font,
        cjk_font: Option<&Font>,
        cell_width: f32,
        line: usize,
    ) -> f32 {
        let text = self.line_text(line);
        shaper.measure(text, primary_font, cjk_font, cell_width)
    }

    pub fn insert_at_cursor(&mut self, text: &str) {
        let mut lines: Vec<String> = self.text.split('\n').map(|l| l.to_owned()).collect();
        let line = self.cursor_line as usize;
        if line >= lines.len() {
            lines.push(String::new());
        }
        let column = self.cursor_column as usize;
        let current = lines[line].chars().collect::<Vec<_>>();
        let char_index = column_to_char_index(&current, column);
        let mut new_line: String = current[..char_index.min(current.len())].iter().collect();
        new_line.push_str(text);
        new_line.extend(&current[char_index.min(current.len())..]);
        lines[line] = new_line;
        self.text = lines.join("\n");
        self.cursor_column = (column + text.chars().count()) as u32;
    }

    pub fn delete_at_cursor(&mut self) {
        let mut lines: Vec<String> = self.text.split('\n').map(|l| l.to_owned()).collect();
        let line = self.cursor_line as usize;
        if line >= lines.len() {
            return;
        }
        let column = self.cursor_column as usize;
        let current = lines[line].chars().collect::<Vec<_>>();
        let char_index = column_to_char_index(&current, column).min(current.len());
        if char_index < current.len() {
            let mut new_line: String = current[..char_index].iter().collect();
            new_line.extend(&current[char_index + 1..]);
            lines[line] = new_line;
            self.text = lines.join("\n");
        } else if line + 1 < lines.len() {
            lines[line] = format!("{}{}", current.iter().collect::<String>(), lines[line + 1]);
            lines.remove(line + 1);
            self.text = lines.join("\n");
        }
    }

    pub fn newline_at_cursor(&mut self) {
        let mut lines: Vec<String> = self.text.split('\n').map(|l| l.to_owned()).collect();
        let line = self.cursor_line as usize;
        if line >= lines.len() {
            lines.push(String::new());
        }
        let column = self.cursor_column as usize;
        let current = lines[line].chars().collect::<Vec<_>>();
        let char_index = column_to_char_index(&current, column).min(current.len());
        let left: String = current[..char_index].iter().collect();
        let right: String = current[char_index..].iter().collect();
        lines[line] = left;
        lines.insert(line + 1, right);
        self.text = lines.join("\n");
        self.cursor_line += 1;
        self.cursor_column = 0;
    }

    pub fn backspace_at_cursor(&mut self) {
        let mut lines: Vec<String> = self.text.split('\n').map(|l| l.to_owned()).collect();
        let line = self.cursor_line as usize;
        if line >= lines.len() {
            return;
        }
        let column = self.cursor_column as usize;
        let current = lines[line].chars().collect::<Vec<_>>();
        let char_index = column_to_char_index(&current, column).min(current.len());
        if char_index > 0 {
            let prev = char_width_at(&current, char_index - 1) as u32;
            let mut new_line: String = current[..char_index - 1].iter().collect();
            new_line.extend(&current[char_index..]);
            lines[line] = new_line;
            self.text = lines.join("\n");
            self.cursor_column = self.cursor_column.saturating_sub(prev);
        } else if line > 0 {
            let upper = lines[line - 1].chars().collect::<Vec<_>>();
            let joined = format!("{}{}", upper.iter().collect::<String>(), current.iter().collect::<String>());
            lines[line - 1] = joined;
            lines.remove(line);
            self.text = lines.join("\n");
            self.cursor_line -= 1;
            self.cursor_column = upper.len() as u32;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_column > 0 {
            self.cursor_column -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            let line = self.line_text(self.cursor_line as usize);
            self.cursor_column = column_width_of_line(line);
        }
    }

    pub fn move_cursor_right(&mut self) {
        let line = self.line_text(self.cursor_line as usize);
        let width = column_width_of_line(line);
        if self.cursor_column < width {
            self.cursor_column += 1;
        } else if self.cursor_line + 1 < self.line_count() as u32 {
            self.cursor_line += 1;
            self.cursor_column = 0;
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor_line + 1 < self.line_count() as u32 {
            self.cursor_line += 1;
        }
    }
}

fn column_to_char_index(chars: &[char], column: usize) -> usize {
    let mut width = 0usize;
    for (index, ch) in chars.iter().enumerate() {
        if width >= column {
            return index;
        }
        width += char_width_kind(*ch as u32) as usize;
    }
    chars.len()
}

fn char_width_at(chars: &[char], index: usize) -> u8 {
    chars
        .get(index)
        .map(|ch| char_width_kind(*ch as u32))
        .unwrap_or(1)
}

fn column_width_of_line(line: &str) -> u32 {
    line.chars()
        .map(|ch| char_width_kind(ch as u32) as u32)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::EditorBuffer;
    use crate::render::render_font::Font;
    use crate::render::render_shape_text::TextShaper;

    #[test]
    fn caret_x_accounts_for_cjk_double_width() {
        let font = Font::embedded();
        let shaper = TextShaper::new(true);
        let mut buffer = EditorBuffer::new(0, String::new(), "你好ab".to_owned());
        buffer.cursor_line = 0;
        buffer.cursor_column = 4;
        let x = buffer.caret_x(&shaper, &font, None, 10.0);
        assert!((x - 40.0).abs() < 0.01, "4 half-width cells -> 40px, got {x}");
    }

    #[test]
    fn insert_and_backspace() {
        let mut buffer = EditorBuffer::new(0, String::new(), "abc".to_owned());
        buffer.cursor_column = 1;
        buffer.insert_at_cursor("X");
        assert_eq!(buffer.text, "aXbc");
        buffer.backspace_at_cursor();
        assert_eq!(buffer.text, "abc");
    }

    #[test]
    fn newline_splits_lines() {
        let mut buffer = EditorBuffer::new(0, String::new(), "ab".to_owned());
        buffer.cursor_column = 1;
        buffer.newline_at_cursor();
        assert_eq!(buffer.text, "a\nb");
        assert_eq!(buffer.cursor_line, 1);
        assert_eq!(buffer.cursor_column, 0);
    }

    #[test]
    fn column_at_x_roundtrips() {
        let font = Font::embedded();
        let shaper = TextShaper::new(true);
        let buffer = EditorBuffer::new(0, String::new(), "你ab".to_owned());
        let x = buffer.caret_x(&shaper, &font, None, 12.0);
        let column = buffer.column_at_x(&shaper, &font, None, 12.0, x);
        assert_eq!(column, 0);
    }
}
