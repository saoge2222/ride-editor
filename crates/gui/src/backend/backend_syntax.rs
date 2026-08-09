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
    pub start: usize,
    pub end: usize,
    pub kind: TokenKind,
}

#[derive(Clone, Debug)]
pub enum SyntaxEvent {
    Tokens { uri: String, tokens: Vec<SyntaxToken> },
}
