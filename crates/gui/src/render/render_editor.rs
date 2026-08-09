use crate::component::component_layout::Rect;
use crate::vulkano_base::vulkano_base_render_loop::CommandBufferBuilder;
use crate::widgets::widgets_editor::EditorBuffer;
use super::render_draw::DrawList;
use super::render_font::Font;
use super::render_shape_text::TextShaper;
use super::render_text::TextRenderer;

const CARET_WIDTH: f32 = 2.0;
const DEFAULT_PADDING: f32 = 8.0;
const DEFAULT_LINE_HEIGHT_MULTIPLIER: f32 = 1.5;
const DEFAULT_GUTTER_COLUMNS: u32 = 4;

pub struct EditorColors {
    pub background: [f32; 4],
    pub text: [f32; 4],
    pub gutter: [f32; 4],
    pub line_number: [f32; 4],
    pub caret: [f32; 4],
}

impl EditorColors {
    pub const fn default() -> Self {
        Self {
            background: [0.10, 0.10, 0.12, 1.0],
            text: [0.92, 0.92, 0.95, 1.0],
            gutter: [0.14, 0.14, 0.16, 1.0],
            line_number: [0.45, 0.45, 0.50, 1.0],
            caret: [0.95, 0.95, 0.98, 1.0],
        }
    }
}

pub struct EditorView {
    pub padding: f32,
    pub line_height: f32,
    pub gutter_columns: u32,
}

impl EditorView {
    pub fn new(cell_width: f32) -> Self {
        Self {
            padding: DEFAULT_PADDING,
            line_height: cell_width * DEFAULT_LINE_HEIGHT_MULTIPLIER,
            gutter_columns: DEFAULT_GUTTER_COLUMNS,
        }
    }

    fn gutter_width(&self, cell_width: f32) -> f32 {
        self.gutter_columns as f32 * cell_width
    }

    pub fn background(
        &self,
        draw_list: &mut DrawList,
        buffer: &EditorBuffer,
        shaper: &TextShaper,
        primary_font: &Font,
        cjk_font: Option<&Font>,
        cell_width: f32,
        bounds: Rect,
        colors: &EditorColors,
    ) {
        draw_list.rect(bounds.x, bounds.y, bounds.width, bounds.height, colors.background);
        let gutter_width = self.gutter_width(cell_width);
        draw_list.rect(bounds.x, bounds.y, gutter_width, bounds.height, colors.gutter);

        let scroll = self.scroll_of(buffer, bounds);
        let caret_x = buffer.caret_x(shaper, primary_font, cjk_font, cell_width);
        let caret_y = bounds.y + self.padding + (buffer.cursor_line as usize - scroll) as f32 * self.line_height;
        draw_list.rect(
            bounds.x + gutter_width + self.padding + caret_x,
            caret_y,
            CARET_WIDTH,
            self.line_height,
            colors.caret,
        );
    }

    pub fn draw_text(
        &self,
        builder: &mut CommandBufferBuilder,
        extent: [u32; 2],
        text: &mut TextRenderer,
        buffer: &EditorBuffer,
        cell_width: f32,
        bounds: Rect,
        colors: &EditorColors,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let gutter_width = self.gutter_width(cell_width);
        let line_count = buffer.line_count();
        let visible = (bounds.height / self.line_height).ceil() as usize + 1;
        let scroll = self.scroll_of(buffer, bounds);

        for line in scroll..(scroll + visible).min(line_count) {
            let y = bounds.y + self.padding + (line - scroll) as f32 * self.line_height;
            let number = (line + 1).to_string();
            let number_x = bounds.x + gutter_width - number.len() as f32 * cell_width;
            text.draw(builder, extent, number_x, y, &number, colors.line_number)?;

            let line_text = buffer.line_text(line);
            let text_x = bounds.x + gutter_width + self.padding;
            text.draw(builder, extent, text_x, y, line_text, colors.text)?;
        }
        Ok(())
    }

    fn scroll_of(&self, buffer: &EditorBuffer, bounds: Rect) -> usize {
        let visible = (bounds.height / self.line_height).ceil() as usize + 1;
        (buffer.cursor_line as usize).saturating_sub(visible / 2)
    }
}
