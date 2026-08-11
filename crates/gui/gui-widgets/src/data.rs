use gpui::SharedString;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LspRange {
    pub start: LspPosition,
    pub end: LspPosition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LspDiagnostic {
    pub range: LspRange,
    pub message: SharedString,
    pub severity: DiagnosticSeverity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Struct,
    Class,
    Module,
    Method,
    Variable,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentSymbol {
    pub name: SharedString,
    pub kind: SymbolKind,
    pub range: LspRange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Keyword,
    Identifier,
    Number,
    String,
    Comment,
    Operator,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyntaxToken {
    pub start: u32,
    pub end: u32,
    pub kind: TokenKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitStatus {
    Unmodified,
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitFileState {
    pub path: SharedString,
    pub status: GitStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GitLineState {
    pub line: u32,
    pub status: GitStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Breakpoint {
    pub line: u32,
}
