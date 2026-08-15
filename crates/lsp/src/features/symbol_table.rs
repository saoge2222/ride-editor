use serde::{Deserialize, Serialize};

use crate::server::sync::Document;
use crate::types::position::{Position, Range};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(from = "u32", into = "u32")]
pub enum SymbolKind {
    Function,
    Struct,
    Class,
    Module,
    Method,
    Variable,
    Other,
}

impl From<u32> for SymbolKind {
    fn from(value: u32) -> Self {
        match value {
            12 => SymbolKind::Function,
            23 => SymbolKind::Struct,
            5 => SymbolKind::Class,
            2 => SymbolKind::Module,
            6 => SymbolKind::Method,
            13 => SymbolKind::Variable,
            _ => SymbolKind::Other,
        }
    }
}

impl From<SymbolKind> for u32 {
    fn from(kind: SymbolKind) -> Self {
        match kind {
            SymbolKind::Function => 12,
            SymbolKind::Struct => 23,
            SymbolKind::Class => 5,
            SymbolKind::Module => 2,
            SymbolKind::Method => 6,
            SymbolKind::Variable => 13,
            SymbolKind::Other => 1,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DocumentSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub range: Range,
}

pub trait SymbolTable {
    fn build(&mut self, document: &Document) -> Vec<DocumentSymbol>;
    fn lookup(&mut self, document: &Document, name: &str) -> Option<Range>;
}

pub struct DefaultSymbolTable;

impl SymbolTable for DefaultSymbolTable {
    fn build(&mut self, document: &Document) -> Vec<DocumentSymbol> {
        let mut symbols = Vec::new();
        for (line_index, line) in document.text.lines().enumerate() {
            let trimmed = line.trim_start();
            if let Some((kind, name)) = match_symbol(trimmed) {
                let char_len = line.chars().count();
                symbols.push(DocumentSymbol {
                    name,
                    kind,
                    range: Range {
                        start: Position {
                            line: line_index as u32,
                            character: 0,
                        },
                        end: Position {
                            line: line_index as u32,
                            character: char_len as u32,
                        },
                    },
                });
            }
        }
        symbols
    }

    fn lookup(&mut self, document: &Document, name: &str) -> Option<Range> {
        self.build(document)
            .iter()
            .find(|symbol| symbol.name == name)
            .map(|symbol| symbol.range)
    }
}

fn match_symbol(trimmed: &str) -> Option<(SymbolKind, String)> {
    let patterns: &[(&str, SymbolKind)] = &[
        ("fn ", SymbolKind::Function),
        ("struct ", SymbolKind::Struct),
        ("enum ", SymbolKind::Other),
        ("trait ", SymbolKind::Other),
        ("impl ", SymbolKind::Other),
        ("mod ", SymbolKind::Module),
        ("let ", SymbolKind::Variable),
        ("const ", SymbolKind::Variable),
        ("static ", SymbolKind::Variable),
    ];
    for (prefix, kind) in patterns {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            let name = rest
                .split(|c: char| c.is_whitespace() || c == '(' || c == '<' || c == ';')
                .next()
                .unwrap_or("");
            if !name.is_empty() {
                return Some((*kind, name.to_string()));
            }
        }
    }
    None
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
    fn builds_symbols_from_rust_lines() {
        let mut table = DefaultSymbolTable;
        let document = doc(
            "mod foo;\nfn main() {}\nstruct Point {\nlet x = 1;\nfn helper() {}\n",
        );
        let symbols = table.build(&document);
        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"foo"));
        assert!(names.contains(&"main"));
        assert!(names.contains(&"Point"));
        assert!(names.contains(&"x"));
        assert!(names.contains(&"helper"));
        let main = symbols.iter().find(|s| s.name == "main").unwrap();
        assert_eq!(main.kind, SymbolKind::Function);
        assert_eq!(main.range.start.line, 1);
    }

    #[test]
    fn ignores_non_symbol_lines() {
        let mut table = DefaultSymbolTable;
        let document = doc("fn main() {\n    println!(\"hi\");\n}");
        let symbols = table.build(&document);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "main");
    }

    #[test]
    fn symbol_kind_numeric_mapping() {
        assert_eq!(u32::from(SymbolKind::Function), 12);
        assert_eq!(u32::from(SymbolKind::Struct), 23);
        assert_eq!(SymbolKind::from(12u32), SymbolKind::Function);
        assert_eq!(SymbolKind::from(99u32), SymbolKind::Other);
    }
}
