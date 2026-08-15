use crate::server::sync::Document;
use crate::types::completion::{CompletionItem, CompletionItemKind};
use crate::types::position::Position;

pub const MAX_COMPLETION_ITEMS: usize = 32;

pub const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
    "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
    "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true",
    "type", "unsafe", "use", "where", "while",
];

pub trait CompletionBackend {
    fn complete(
        &mut self,
        document: &Document,
        position: &Position,
        trigger: Option<&str>,
    ) -> Vec<CompletionItem>;
}

pub struct DefaultCompletion;

impl CompletionBackend for DefaultCompletion {
    fn complete(
        &mut self,
        document: &Document,
        position: &Position,
        trigger: Option<&str>,
    ) -> Vec<CompletionItem> {
        let _ = trigger;
        let prefix = word_prefix_at(document, position);
        let items: Vec<CompletionItem> = RUST_KEYWORDS
            .iter()
            .filter(|keyword| keyword.starts_with(&prefix))
            .take(MAX_COMPLETION_ITEMS)
            .map(|keyword| CompletionItem {
                label: keyword.to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("keyword".to_string()),
                documentation: None,
                insert_text: Some(keyword.to_string()),
                data: None,
            })
            .collect();
        if items.is_empty() && !prefix.is_empty() {
            vec![CompletionItem {
                label: prefix,
                kind: CompletionItemKind::Keyword,
                detail: Some("keyword".to_string()),
                documentation: None,
                insert_text: None,
                data: None,
            }]
        } else {
            items
        }
    }
}

fn word_prefix_at(document: &Document, position: &Position) -> String {
    let line = document
        .text
        .lines()
        .nth(position.line as usize)
        .unwrap_or("");
    let end = line.chars().take(position.character as usize).collect::<String>();
    let mut prefix = String::new();
    for ch in end.chars().rev() {
        if ch.is_alphanumeric() || ch == '_' {
            prefix.insert(0, ch);
        } else {
            break;
        }
    }
    prefix
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
    fn prefix_filters_keywords() {
        let mut backend = DefaultCompletion;
        let document = doc("let x = f");
        let items = backend.complete(
            &document,
            &Position {
                line: 0,
                character: 9,
            },
            None,
        );
        assert!(items.iter().any(|i| i.label == "fn"));
        assert!(items.iter().all(|i| i.label.starts_with('f')));
    }

    #[test]
    fn empty_prefix_returns_full_table_capped() {
        let mut backend = DefaultCompletion;
        let document = doc("");
        let items = backend.complete(
            &document,
            &Position {
                line: 0,
                character: 0,
            },
            None,
        );
        assert!(!items.is_empty());
        assert!(items.len() <= MAX_COMPLETION_ITEMS);
    }

    #[test]
    fn mid_word_prefix_only_scans_backwards() {
        let mut backend = DefaultCompletion;
        let document = doc("let x = 1; f");
        let items = backend.complete(
            &document,
            &Position {
                line: 0,
                character: 11,
            },
            None,
        );
        assert!(items.iter().any(|i| i.label == "fn"));
    }
}
