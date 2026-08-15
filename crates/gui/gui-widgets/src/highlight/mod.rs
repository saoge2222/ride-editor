use crate::data::SyntaxToken;
use gpui::Hsla;

#[derive(Clone, Debug)]
pub struct HighlightRange {
    pub start: usize,
    pub end: usize,
    pub color: Hsla,
}

pub struct HighlightService {}

impl HighlightService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn highlight(&self, _text: &str, _tokens: &[SyntaxToken]) -> Vec<HighlightRange> {
        Vec::new()
    }
}

impl Default for HighlightService {
    fn default() -> Self {
        Self::new()
    }
}
