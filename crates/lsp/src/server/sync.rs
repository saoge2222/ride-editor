use std::collections::HashMap;

use crate::document::{DocumentPayload, DocumentSnapshot, DocumentSource};
use crate::types::position::Position;
use crate::types::text_document::{
    TextDocumentContentChangeEvent, TextDocumentSyncKind,
};

pub struct Document {
    pub uri: String,
    pub language_id: String,
    pub version: i32,
    pub text: String,
}

pub struct DocumentStore {
    documents: HashMap<String, Document>,
    sync_kind: TextDocumentSyncKind,
}

impl DocumentStore {
    pub fn new(sync_kind: TextDocumentSyncKind) -> Self {
        DocumentStore {
            documents: HashMap::new(),
            sync_kind,
        }
    }

    pub fn open(&mut self, payload: DocumentPayload) -> Result<(), String> {
        self.documents.insert(
            payload.uri.clone(),
            Document {
                uri: payload.uri,
                language_id: payload.language_id,
                version: payload.version,
                text: payload.text,
            },
        );
        Ok(())
    }

    pub fn apply_change_full(
        &mut self,
        uri: &str,
        version: i32,
        text: &str,
    ) -> Result<(), String> {
        let document = self.documents.get_mut(uri).ok_or("文档未打开")?;
        document.text = text.to_string();
        document.version = version;
        Ok(())
    }

    pub fn apply_change_incremental(
        &mut self,
        uri: &str,
        version: i32,
        changes: &[TextDocumentContentChangeEvent],
    ) -> Result<(), String> {
        let document = self.documents.get_mut(uri).ok_or("文档未打开")?;
        for change in changes {
            let range = match &change.range {
                Some(range) => *range,
                None => return Err("增量变更缺少 range".to_string()),
            };
            let new_text = if change.text.is_empty() {
                String::new()
            } else {
                change.text.clone()
            };
            document.text = Self::apply_single_change(&document.text, range, &new_text);
        }
        document.version = version;
        Ok(())
    }

    pub fn close(&mut self, uri: &str) -> Result<(), String> {
        self.documents
            .remove(uri)
            .map(|_| ())
            .ok_or_else(|| "文档未打开".to_string())
    }

    pub fn save(&mut self, uri: &str) -> Result<(), String> {
        self.documents
            .get(uri)
            .map(|_| ())
            .ok_or_else(|| "文档未打开".to_string())
    }

    pub fn get(&self, uri: &str) -> Option<&Document> {
        self.documents.get(uri)
    }

    pub fn snapshot(&self, uri: &str) -> Option<DocumentSnapshot> {
        self.documents.get(uri).map(|doc| DocumentSnapshot {
            uri: doc.uri.clone(),
            version: doc.version,
            text: doc.text.clone(),
        })
    }

    pub fn is_open(&self, uri: &str) -> bool {
        self.documents.contains_key(uri)
    }

    fn apply_single_change(text: &str, range: crate::types::position::Range, new_text: &str) -> String {
        let start = Self::position_to_offset(text, range.start);
        let end = Self::position_to_offset(text, range.end);
        if start > end || end > text.len() {
            return text.to_string();
        }
        let mut result = String::with_capacity(text.len() + new_text.len());
        result.push_str(&text[..start]);
        result.push_str(new_text);
        result.push_str(&text[end..]);
        result
    }

    fn position_to_offset(text: &str, pos: Position) -> usize {
        let mut current_line = 0u32;
        let mut offset = 0usize;
        for (idx, byte) in text.bytes().enumerate() {
            if current_line == pos.line {
                offset = idx;
                break;
            }
            if byte == b'\n' {
                current_line += 1;
                offset = idx + 1;
            }
        }
        if current_line < pos.line {
            return text.len();
        }
        let line_start = offset;
        let mut char_count = 0u32;
        for (idx, _) in text[line_start..].char_indices() {
            if char_count == pos.character {
                return line_start + idx;
            }
            char_count += 1;
        }
        if char_count <= pos.character {
            return text.len();
        }
        line_start
    }
}

impl DocumentSource for DocumentStore {
    fn open(&mut self, payload: DocumentPayload) -> Result<(), String> {
        DocumentStore::open(self, payload)
    }

    fn change(
        &mut self,
        uri: &str,
        version: i32,
        changes: &[TextDocumentContentChangeEvent],
    ) -> Result<(), String> {
        match self.sync_kind {
            TextDocumentSyncKind::Full => {
                let text = changes
                    .last()
                    .map(|c| c.text.clone())
                    .ok_or("全量变更缺少内容")?;
                self.apply_change_full(uri, version, &text)
            }
            TextDocumentSyncKind::Incremental => {
                self.apply_change_incremental(uri, version, changes)
            }
            TextDocumentSyncKind::None => Err("未启用文档同步".to_string()),
        }
    }

    fn close(&mut self, uri: &str) -> Result<(), String> {
        DocumentStore::close(self, uri)
    }

    fn save(&mut self, uri: &str) -> Result<(), String> {
        DocumentStore::save(self, uri)
    }

    fn document(&self, uri: &str) -> Option<DocumentSnapshot> {
        DocumentStore::snapshot(self, uri)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::position::Position;

    fn pos(line: u32, character: u32) -> Position {
        Position { line, character }
    }

    fn open_store(text: &str) -> DocumentStore {
        let mut store = DocumentStore::new(TextDocumentSyncKind::Incremental);
        store
            .open(DocumentPayload {
                uri: "file:///test.rs".to_string(),
                language_id: "rust".to_string(),
                version: 1,
                text: text.to_string(),
            })
            .expect("打开文档失败");
        store
    }

    #[test]
    fn full_change_replaces_text() {
        let mut store = open_store("old text");
        store
            .apply_change_full("file:///test.rs", 2, "new text")
            .expect("全量变更失败");
        assert_eq!(
            store.snapshot("file:///test.rs").expect("无快照").text,
            "new text"
        );
        assert_eq!(store.snapshot("file:///test.rs").unwrap().version, 2);
    }

    #[test]
    fn incremental_replace_middle() {
        let mut store = open_store("fn foo() {}");
        let change = TextDocumentContentChangeEvent {
            range: Some(crate::types::position::Range {
                start: pos(0, 3),
                end: pos(0, 6),
            }),
            range_length: None,
            text: "bar".to_string(),
        };
        store
            .apply_change_incremental("file:///test.rs", 2, &[change])
            .expect("增量变更失败");
        assert_eq!(
            store.snapshot("file:///test.rs").unwrap().text,
            "fn bar() {}"
        );
    }

    #[test]
    fn incremental_insert_at_start() {
        let mut store = open_store("foo()");
        let change = TextDocumentContentChangeEvent {
            range: Some(crate::types::position::Range {
                start: pos(0, 0),
                end: pos(0, 0),
            }),
            range_length: None,
            text: "pub ".to_string(),
        };
        store
            .apply_change_incremental("file:///test.rs", 2, &[change])
            .expect("增量变更失败");
        assert_eq!(store.snapshot("file:///test.rs").unwrap().text, "pub foo()");
    }

    #[test]
    fn incremental_delete_at_end() {
        let mut store = open_store("hello world");
        let change = TextDocumentContentChangeEvent {
            range: Some(crate::types::position::Range {
                start: pos(0, 5),
                end: pos(0, 11),
            }),
            range_length: None,
            text: String::new(),
        };
        store
            .apply_change_incremental("file:///test.rs", 2, &[change])
            .expect("增量变更失败");
        assert_eq!(store.snapshot("file:///test.rs").unwrap().text, "hello");
    }

    #[test]
    fn incremental_across_lines() {
        let mut store = open_store("fn a() {}\nfn b() {}\n");
        let change = TextDocumentContentChangeEvent {
            range: Some(crate::types::position::Range {
                start: pos(0, 6),
                end: pos(1, 0),
            }),
            range_length: None,
            text: " -> i32".to_string(),
        };
        store
            .apply_change_incremental("file:///test.rs", 2, &[change])
            .expect("增量变更失败");
        assert_eq!(
            store.snapshot("file:///test.rs").unwrap().text,
            "fn a() -> i32fn b() {}\n"
        );
    }

    #[test]
    fn multiple_changes_applied_in_order() {
        let mut store = open_store("abc");
        let change1 = TextDocumentContentChangeEvent {
            range: Some(crate::types::position::Range {
                start: pos(0, 0),
                end: pos(0, 1),
            }),
            range_length: None,
            text: "x".to_string(),
        };
        let change2 = TextDocumentContentChangeEvent {
            range: Some(crate::types::position::Range {
                start: pos(0, 2),
                end: pos(0, 3),
            }),
            range_length: None,
            text: "y".to_string(),
        };
        store
            .apply_change_incremental("file:///test.rs", 2, &[change1, change2])
            .expect("增量变更失败");
        assert_eq!(store.snapshot("file:///test.rs").unwrap().text, "xby");
    }

    #[test]
    fn close_and_reopen() {
        let mut store = open_store("text");
        store.close("file:///test.rs").expect("关闭失败");
        assert!(!store.is_open("file:///test.rs"));
        assert!(store.close("file:///test.rs").is_err());
    }

    #[test]
    fn change_unknown_document_errors() {
        let mut store = open_store("text");
        let change = TextDocumentContentChangeEvent {
            range: Some(crate::types::position::Range {
                start: pos(0, 0),
                end: pos(0, 1),
            }),
            range_length: None,
            text: "x".to_string(),
        };
        assert!(store
            .apply_change_incremental("file:///missing.rs", 2, &[change])
            .is_err());
    }

    #[test]
    fn full_sync_via_document_source() {
        let mut store = DocumentStore::new(TextDocumentSyncKind::Full);
        store
            .open(DocumentPayload {
                uri: "file:///a.rs".to_string(),
                language_id: "rust".to_string(),
                version: 1,
                text: "old".to_string(),
            })
            .expect("打开失败");
        let change = TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "brand new".to_string(),
        };
        store
            .change("file:///a.rs", 2, &[change])
            .expect("同步失败");
        assert_eq!(store.snapshot("file:///a.rs").unwrap().text, "brand new");
    }

    #[test]
    fn position_beyond_text_end_clamps() {
        let mut store = open_store("fn a() {}\nfn b() {}\n");
        let change = TextDocumentContentChangeEvent {
            range: Some(crate::types::position::Range {
                start: pos(99, 0),
                end: pos(99, 5),
            }),
            range_length: None,
            text: "end".to_string(),
        };
        store
            .apply_change_incremental("file:///test.rs", 2, &[change])
            .expect("增量变更失败");
        assert_eq!(store.snapshot("file:///test.rs").unwrap().text, "fn a() {}\nfn b() {}\nend");
    }
}
