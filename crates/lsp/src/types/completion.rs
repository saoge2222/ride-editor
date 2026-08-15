use serde::{Deserialize, Serialize};

use crate::types::position::Position;
use crate::types::text_document::TextDocumentIdentifier;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CompletionContext {
    #[serde(rename = "triggerKind")]
    pub trigger_kind: u32,
    #[serde(rename = "triggerCharacter", skip_serializing_if = "Option::is_none", default)]
    pub trigger_character: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CompletionParams {
    #[serde(rename = "textDocument")]
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub context: Option<CompletionContext>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(from = "u32", into = "u32")]
pub enum CompletionItemKind {
    Function,
    Class,
    Method,
    Variable,
    Module,
    Keyword,
    Other,
}

impl From<u32> for CompletionItemKind {
    fn from(value: u32) -> Self {
        match value {
            3 => CompletionItemKind::Function,
            5 => CompletionItemKind::Class,
            6 => CompletionItemKind::Method,
            4 => CompletionItemKind::Variable,
            9 => CompletionItemKind::Module,
            14 => CompletionItemKind::Keyword,
            _ => CompletionItemKind::Other,
        }
    }
}

impl From<CompletionItemKind> for u32 {
    fn from(kind: CompletionItemKind) -> Self {
        match kind {
            CompletionItemKind::Function => 3,
            CompletionItemKind::Class => 5,
            CompletionItemKind::Method => 6,
            CompletionItemKind::Variable => 4,
            CompletionItemKind::Module => 9,
            CompletionItemKind::Keyword => 14,
            CompletionItemKind::Other => 255,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionItemKind,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub documentation: Option<String>,
    #[serde(rename = "insertText", skip_serializing_if = "Option::is_none", default)]
    pub insert_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub data: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CompletionList {
    pub is_incomplete: bool,
    pub items: Vec<CompletionItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum CompletionResponse {
    List(CompletionList),
    Items(Vec<CompletionItem>),
}
