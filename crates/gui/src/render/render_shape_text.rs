use super::render_font::Font;
use super::render_font_gsub::{apply_gsub, parse_gsub};
use super::render_font_ligature;

const FULLWIDTH_RANGES: &[(u32, u32)] = &[
    (0x1100, 0x115F),
    (0x2329, 0x232A),
    (0x2E80, 0x303E),
    (0x3040, 0x33BF),
    (0x3400, 0x4DBF),
    (0x4E00, 0x9FFF),
    (0xA000, 0xA4CF),
    (0xAC00, 0xD7A3),
    (0xF900, 0xFAFF),
    (0xFE10, 0xFE19),
    (0xFE30, 0xFE6F),
    (0xFF01, 0xFF60),
    (0xFFE0, 0xFFE6),
    (0x20000, 0x2FFFD),
    (0x30000, 0x3FFFD),
];

pub fn is_fullwidth(cp: u32) -> bool {
    if cp < 0x1100 {
        return false;
    }
    let mut lo = 0usize;
    let mut hi = FULLWIDTH_RANGES.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let (start, end) = FULLWIDTH_RANGES[mid];
        if cp < start {
            hi = mid;
        } else if cp > end {
            lo = mid + 1;
        } else {
            return true;
        }
    }
    false
}

pub fn char_width_kind(cp: u32) -> u8 {
    if is_fullwidth(cp) { 2 } else { 1 }
}

pub struct ShapedToken {
    pub glyph_id: u16,
    pub font_index: usize,
    pub advance_px: f32,
    pub width_cells: u8,
    pub codepoint: u32,
}

pub struct ShapedText {
    pub tokens: Vec<ShapedToken>,
    pub cell_width: f32,
}

#[derive(Clone, Copy)]
pub struct TextShaper {
    ligature_enabled: bool,
}

impl TextShaper {
    pub fn new(ligature_enabled: bool) -> Self {
        Self { ligature_enabled }
    }

    pub fn shape(
        &self,
        text: &str,
        primary_font: &Font,
        cjk_font: Option<&Font>,
        cell_width: f32,
    ) -> ShapedText {
        let use_gsub = self.ligature_enabled && primary_font.has_gsub();
        let chars: Vec<char> = if self.ligature_enabled {
            render_font_ligature::apply_ligatures(text, |ch| primary_font.glyph_index(ch).is_some())
        } else {
            text.chars().collect()
        };

        let mut base: Vec<(u32, u16, usize, u8)> = Vec::with_capacity(chars.len());
        for ch in chars {
            let cp = ch as u32;
            let width = char_width_kind(cp);
            if let Some(glyph) = primary_font.glyph_index_full(cp) {
                base.push((cp, glyph, 0, width));
            } else if let Some(cjk) = cjk_font {
                if let Some(glyph) = cjk.glyph_index_full(cp) {
                    base.push((cp, glyph, 1, width));
                } else {
                    base.push((cp, primary_font.glyph_index('?').unwrap_or(0), 0, width));
                }
            } else {
                base.push((cp, primary_font.glyph_index('?').unwrap_or(0), 0, width));
            }
        }

        let mut tokens = Vec::new();
        let mut index = 0;
        while index < base.len() {
            let font_index = base[index].2;
            let mut run_end = index;
            while run_end < base.len() && base[run_end].2 == font_index {
                run_end += 1;
            }
            if font_index == 0 && use_gsub {
                if let Some(table) = primary_font.gsub_table() {
                    if let Some(gsub) = parse_gsub(primary_font.raw_data(), table) {
                        let glyph_ids: Vec<u16> = base[index..run_end].iter().map(|b| b.1).collect();
                        let shaped = apply_gsub(primary_font.raw_data(), &gsub, &glyph_ids);
                        let mut offset = 0;
                        for (glyph_id, consumed) in shaped {
                            let consumed = consumed.max(1);
                            let end = (offset + consumed).min(run_end - index);
                            let cells: u8 = base[index + offset..index + end]
                                .iter()
                                .map(|b| b.3)
                                .sum();
                            tokens.push(ShapedToken {
                                glyph_id,
                                font_index,
                                advance_px: cell_width * cells as f32,
                                width_cells: cells.max(1),
                                codepoint: base[index + offset].0,
                            });
                            offset += consumed;
                        }
                        index = run_end;
                        continue;
                    }
                }
            }
            for item in &base[index..run_end] {
                tokens.push(ShapedToken {
                    glyph_id: item.1,
                    font_index: item.2,
                    advance_px: cell_width * item.3 as f32,
                    width_cells: item.3,
                    codepoint: item.0,
                });
            }
            index = run_end;
        }

        ShapedText { tokens, cell_width }
    }

    pub fn measure(
        &self,
        text: &str,
        primary_font: &Font,
        cjk_font: Option<&Font>,
        cell_width: f32,
    ) -> f32 {
        let shaped = self.shape(text, primary_font, cjk_font, cell_width);
        shaped.tokens.iter().map(|t| t.advance_px).sum()
    }

    pub fn caret_x(
        &self,
        text: &str,
        primary_font: &Font,
        cjk_font: Option<&Font>,
        cell_width: f32,
        char_index: usize,
    ) -> f32 {
        let shaped = self.shape(text, primary_font, cjk_font, cell_width);
        shaped
            .tokens
            .iter()
            .take(char_index.min(shaped.tokens.len()))
            .map(|t| t.advance_px)
            .sum()
    }

    pub fn column_to_x(&self, shaped: &ShapedText, mut column: usize) -> f32 {
        let mut x = 0.0;
        for token in &shaped.tokens {
            if column <= 0 {
                break;
            }
            if column >= token.width_cells as usize {
                x += token.advance_px;
            } else {
                x += token.advance_px * (column as f32 / token.width_cells as f32);
                break;
            }
            column -= token.width_cells as usize;
        }
        x
    }

    pub fn x_to_column(&self, shaped: &ShapedText, x: f32) -> usize {
        let mut column = 0;
        let mut pos = 0.0;
        for token in &shaped.tokens {
            if pos + token.advance_px >= x {
                let within = (x - pos) / token.advance_px.max(0.0001);
                column += (within * token.width_cells as f32).round() as usize;
                return column;
            }
            pos += token.advance_px;
            column += token.width_cells as usize;
        }
        column
    }
}

#[cfg(test)]
mod tests {
    use super::{char_width_kind, is_fullwidth, TextShaper};
    use crate::render::render_font::Font;

    #[test]
    fn maplemono_gsub_ligature_arrow() {
        let font = Font::embedded();
        let shaper = TextShaper::new(true);
        let shaped = shaper.shape("->", &font, None, 12.0);
        assert!(shaped.tokens.len() < 2, "arrow should ligate, got {} tokens", shaped.tokens.len());
    }

    #[test]
    fn maplemono_gsub_ligature_not_equal() {
        let font = Font::embedded();
        let shaper = TextShaper::new(true);
        let shaped = shaper.shape("!=", &font, None, 12.0);
        assert!(shaped.tokens.len() < 2, "not-equal should ligate, got {} tokens", shaped.tokens.len());
    }

    #[test]
    fn maplemono_no_ligature_for_plain_text() {
        let font = Font::embedded();
        let shaper = TextShaper::new(true);
        let shaped = shaper.shape("abc", &font, None, 12.0);
        assert_eq!(shaped.tokens.len(), 3);
    }

    #[test]
    fn latin_is_narrow() {
        assert!(!is_fullwidth('a' as u32));
        assert!(!is_fullwidth('A' as u32));
        assert!(!is_fullwidth('1' as u32));
        assert!(!is_fullwidth(' ' as u32));
    }

    #[test]
    fn cjk_is_fullwidth() {
        assert!(is_fullwidth('\u{4E2D}' as u32));
        assert!(is_fullwidth('\u{6587}' as u32));
        assert!(is_fullwidth('\u{4F60}' as u32));
        assert!(is_fullwidth('\u{597D}' as u32));
        assert!(is_fullwidth('\u{4E16}' as u32));
        assert!(is_fullwidth('\u{754C}' as u32));
    }

    #[test]
    fn hiragana_is_fullwidth() {
        assert!(is_fullwidth('\u{3042}' as u32));
        assert!(is_fullwidth('\u{3053}' as u32));
    }

    #[test]
    fn hangul_is_fullwidth() {
        assert!(is_fullwidth('\u{AC00}' as u32));
        assert!(is_fullwidth('\u{D7A3}' as u32));
    }

    #[test]
    fn fullwidth_forms_are_fullwidth() {
        assert!(is_fullwidth('\u{FF01}' as u32));
    }

    #[test]
    fn arrow_is_narrow() {
        assert!(!is_fullwidth('\u{2192}' as u32));
    }

    #[test]
    fn char_width_kind_returns_2_for_cjk() {
        assert_eq!(char_width_kind('\u{4E2D}' as u32), 2);
        assert_eq!(char_width_kind('a' as u32), 1);
    }
}

