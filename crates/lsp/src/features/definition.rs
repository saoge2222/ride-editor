use crate::server::sync::Document;
use crate::types::position::{Location, Position, Range};

pub trait DefinitionBackend {
    fn define(&mut self, document: &Document, position: &Position) -> Option<Location>;
}

pub struct DefaultDefinition;

impl DefinitionBackend for DefaultDefinition {
    fn define(&mut self, document: &Document, position: &Position) -> Option<Location> {
        let name = identifier_at(document, position)?;
        find_first_occurrence(document, &name).map(|range| Location {
            uri: document.uri.clone(),
            range,
        })
    }
}

pub fn identifier_at(document: &Document, position: &Position) -> Option<String> {
    let line = document
        .text
        .lines()
        .nth(position.line as usize)
        .unwrap_or("");
    let offset = byte_offset_in_line(line, position.character as usize);
    let bytes = line.as_bytes();
    if offset >= bytes.len() {
        return None;
    }
    if !is_ident_byte(bytes[offset]) {
        return None;
    }
    let mut start = offset;
    while start > 0 && is_ident_byte(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = offset;
    while end < bytes.len() && is_ident_byte(bytes[end]) {
        end += 1;
    }
    Some(line[start..end].to_string())
}

fn find_first_occurrence(document: &Document, name: &str) -> Option<Range> {
    for (line_index, line) in document.text.lines().enumerate() {
        if let Some(byte_pos) = line.find(name) {
            let char_pos = line[..byte_pos].chars().count();
            let char_end = char_pos + name.chars().count();
            return Some(Range {
                start: Position {
                    line: line_index as u32,
                    character: char_pos as u32,
                },
                end: Position {
                    line: line_index as u32,
                    character: char_end as u32,
                },
            });
        }
    }
    None
}

fn byte_offset_in_line(line: &str, character: usize) -> usize {
    line.char_indices()
        .nth(character)
        .map(|(idx, _)| idx)
        .unwrap_or(line.len())
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::sync::Document;

    fn doc(text: &str) -> Document {
        Document {
            uri: "file:///t.rs".to_string(),
            language_id: "rust".to_string(),
            version: 1,
            text: text.to_string(),
        }
    }

    #[test]
    fn finds_definition_of_main() {
        let mut backend = DefaultDefinition;
        let document = doc("fn main() {}\nfn helper() {}\n");
        let location = backend
            .define(
                &document,
                &Position {
                    line: 1,
                    character: 5,
                },
            )
            .expect("应找到定义");
        assert_eq!(location.uri, "file:///t.rs");
        assert_eq!(location.range.start.line, 1);
        assert_eq!(location.range.start.character, 3);
    }

    #[test]
    fn no_identifier_returns_none() {
        let mut backend = DefaultDefinition;
        let document = doc("fn main() {}");
        assert!(backend
            .define(
                &document,
                &Position {
                    line: 0,
                    character: 2,
                }
            )
            .is_none());
    }

    #[test]
    fn unknown_identifier_returns_none() {
        let mut backend = DefaultDefinition;
        let document = doc("fn main() {}");
        assert!(backend
            .define(
                &document,
                &Position {
                    line: 0,
                    character: 7,
                }
            )
            .is_none());
    }

    #[test]
    fn identifier_at_extracts_word() {
        let document = doc("let value = 1;");
        assert_eq!(
            identifier_at(
                &document,
                &Position {
                    line: 0,
                    character: 6,
                }
            ),
            Some("value".to_string())
        );
        assert_eq!(
            identifier_at(
                &document,
                &Position {
                    line: 0,
                    character: 0,
                }
            ),
            Some("let".to_string())
        );
        assert_eq!(
            identifier_at(
                &document,
                &Position {
                    line: 0,
                    character: 2,
                }
            ),
            Some("let".to_string())
        );
    }
}
