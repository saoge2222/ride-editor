use crate::types::text_document::{TextDocumentContentChangeEvent, TextDocumentItem};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentPayload {
    pub uri: String,
    pub language_id: String,
    pub version: i32,
    pub text: String,
}

impl DocumentPayload {
    pub fn from_text_document_item(item: &TextDocumentItem) -> Self {
        DocumentPayload {
            uri: item.uri.clone(),
            language_id: item.language_id.clone(),
            version: item.version,
            text: item.text.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentSnapshot {
    pub uri: String,
    pub version: i32,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DocumentEvent {
    Opened(DocumentPayload),
    Changed {
        uri: String,
        version: i32,
        changes: Vec<TextDocumentContentChangeEvent>,
    },
    Closed { uri: String },
    Saved { uri: String },
}

pub trait DocumentSource {
    fn open(&mut self, payload: DocumentPayload) -> Result<(), String>;
    fn change(
        &mut self,
        uri: &str,
        version: i32,
        changes: &[TextDocumentContentChangeEvent],
    ) -> Result<(), String>;
    fn close(&mut self, uri: &str) -> Result<(), String>;
    fn save(&mut self, uri: &str) -> Result<(), String>;
    fn document(&self, uri: &str) -> Option<DocumentSnapshot>;
}
