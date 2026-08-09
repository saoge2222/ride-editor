#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LspRange {
    pub start: LspPosition,
    pub end: LspPosition,
}

#[derive(Clone, Debug)]
pub struct LspDiagnostic {
    pub range: LspRange,
    pub message: String,
}

#[derive(Clone, Debug)]
pub enum LspEvent {
    Diagnostics { uri: String, diagnostics: Vec<LspDiagnostic> },
    Completion { uri: String, items: Vec<String> },
    Hover { uri: String, text: String },
}
