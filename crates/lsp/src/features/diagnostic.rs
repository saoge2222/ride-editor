use crate::server::sync::Document;
use crate::types::diagnostic::{Diagnostic, DiagnosticSeverity};
use crate::types::position::{Position, Range};

pub trait DiagnosticBackend {
    fn scan(&mut self, document: &Document) -> Vec<Diagnostic>;
}

pub struct BraceDiagnostic;

impl DiagnosticBackend for BraceDiagnostic {
    fn scan(&mut self, document: &Document) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut stack: Vec<(char, Position)> = Vec::new();
        for (line_index, line) in document.text.lines().enumerate() {
            for (char_index, ch) in line.chars().enumerate() {
                let pos = Position {
                    line: line_index as u32,
                    character: char_index as u32,
                };
                match ch {
                    '{' | '(' | '[' => stack.push((ch, pos)),
                    '}' | ')' | ']' => {
                        let opening = match ch {
                            '}' => Some('{'),
                            ')' => Some('('),
                            _ => Some('['),
                        };
                        let matched = stack.pop().map(|(top, _)| top == opening.unwrap());
                        if matched != Some(true) {
                            diagnostics.push(Diagnostic {
                                range: single_char_range(pos),
                                message: format!("unmatched closing bracket '{ch}'"),
                                severity: DiagnosticSeverity::Error,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
        for (ch, pos) in stack {
            diagnostics.push(Diagnostic {
                range: single_char_range(pos),
                message: format!("unclosed opening bracket '{ch}'"),
                severity: DiagnosticSeverity::Warning,
            });
        }
        diagnostics
    }
}

fn single_char_range(pos: Position) -> Range {
    Range {
        start: pos,
        end: Position {
            line: pos.line,
            character: pos.character + 1,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(text: &str) -> Document {
        Document {
            uri: "file:///t.rs".to_string(),
            language_id: "rust".to_string(),
            version: 1,
            text: text.to_string(),
        }
    }

    #[test]
    fn balanced_code_has_no_diagnostics() {
        let mut backend = BraceDiagnostic;
        let document = doc("fn main() {\n    let x = 1;\n}\n");
        assert!(backend.scan(&document).is_empty());
    }

    #[test]
    fn unmatched_closing_is_error() {
        let mut backend = BraceDiagnostic;
        let document = doc("fn main() {}\n}\n");
        let diagnostics = backend.scan(&document);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostics[0].range.start.line, 1);
    }

    #[test]
    fn unclosed_opening_is_warning() {
        let mut backend = BraceDiagnostic;
        let document = doc("fn main() {\n    let x = 1;\n");
        let diagnostics = backend.scan(&document);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Warning);
        assert_eq!(diagnostics[0].range.start.line, 0);
    }

    #[test]
    fn mismatch_bracket_is_error() {
        let mut backend = BraceDiagnostic;
        let document = doc("fn main(] {}\n");
        let diagnostics = backend.scan(&document);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains(']'));
    }
}
