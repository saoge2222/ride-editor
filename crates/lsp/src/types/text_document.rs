use serde::{Deserialize, Serialize};

use crate::types::position::Range;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TextDocumentIdentifier {
    pub uri: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct VersionedTextDocumentIdentifier {
    pub uri: String,
    pub version: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TextDocumentItem {
    pub uri: String,
    #[serde(rename = "languageId")]
    pub language_id: String,
    pub version: i32,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TextDocumentContentChangeEvent {
    pub range: Option<Range>,
    #[serde(rename = "rangeLength", skip_serializing_if = "Option::is_none", default)]
    pub range_length: Option<u32>,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DidOpenTextDocumentParams {
    #[serde(rename = "textDocument")]
    pub text_document: TextDocumentItem,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DidChangeTextDocumentParams {
    #[serde(rename = "textDocument")]
    pub text_document: VersionedTextDocumentIdentifier,
    #[serde(rename = "contentChanges")]
    pub content_changes: Vec<TextDocumentContentChangeEvent>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DidCloseTextDocumentParams {
    #[serde(rename = "textDocument")]
    pub text_document: TextDocumentIdentifier,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DidSaveTextDocumentParams {
    #[serde(rename = "textDocument")]
    pub text_document: TextDocumentIdentifier,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub text: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "u8", into = "u8")]
pub enum TextDocumentSyncKind {
    None,
    Full,
    Incremental,
}

impl From<u8> for TextDocumentSyncKind {
    fn from(value: u8) -> Self {
        match value {
            1 => TextDocumentSyncKind::Full,
            2 => TextDocumentSyncKind::Incremental,
            _ => TextDocumentSyncKind::None,
        }
    }
}

impl From<TextDocumentSyncKind> for u8 {
    fn from(kind: TextDocumentSyncKind) -> Self {
        match kind {
            TextDocumentSyncKind::None => 0,
            TextDocumentSyncKind::Full => 1,
            TextDocumentSyncKind::Incremental => 2,
        }
    }
}
