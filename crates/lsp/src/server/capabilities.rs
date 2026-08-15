use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::text_document::TextDocumentSyncKind;

pub const SERVER_NAME: &str = "ride-lsp";
pub const SERVER_VERSION: &str = "0.1.0";

#[derive(Deserialize, Clone, Debug, Default)]
pub struct ClientCapabilities {
    #[serde(rename = "dynamicRegistration", default)]
    pub dynamic_registration: bool,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct InitializeParams {
    #[serde(rename = "processId", default)]
    pub process_id: Option<u64>,
    #[serde(rename = "clientInfo", default)]
    pub client_info: Option<Value>,
    #[serde(default)]
    pub capabilities: ClientCapabilities,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct CompletionOptions {
    #[serde(rename = "resolveProvider", default)]
    pub resolve_provider: bool,
    #[serde(rename = "triggerCharacters", skip_serializing_if = "Option::is_none", default)]
    pub trigger_characters: Option<Vec<String>>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ServerCapabilities {
    #[serde(rename = "textDocumentSync")]
    pub text_document_sync: TextDocumentSyncKind,
    #[serde(rename = "completionProvider", skip_serializing_if = "Option::is_none", default)]
    pub completion_provider: Option<CompletionOptions>,
    #[serde(rename = "definitionProvider", skip_serializing_if = "Option::is_none", default)]
    pub definition_provider: Option<bool>,
    #[serde(rename = "hoverProvider", skip_serializing_if = "Option::is_none", default)]
    pub hover_provider: Option<bool>,
    #[serde(rename = "referencesProvider", skip_serializing_if = "Option::is_none", default)]
    pub references_provider: Option<bool>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct InitializeResult {
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

pub fn default_server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: TextDocumentSyncKind::Incremental,
        completion_provider: Some(CompletionOptions {
            resolve_provider: true,
            trigger_characters: Some(vec![".".to_string()]),
        }),
        definition_provider: Some(true),
        hover_provider: Some(true),
        references_provider: Some(true),
    }
}

pub fn initialize_result() -> InitializeResult {
    InitializeResult {
        capabilities: default_server_capabilities(),
        server_info: ServerInfo {
            name: SERVER_NAME.to_string(),
            version: SERVER_VERSION.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_incremental_with_completion() {
        let caps = default_server_capabilities();
        assert_eq!(caps.text_document_sync, TextDocumentSyncKind::Incremental);
        let completion = caps.completion_provider.expect("应有补全能力");
        assert!(completion.resolve_provider);
        assert_eq!(completion.trigger_characters, Some(vec![".".to_string()]));
        assert_eq!(caps.definition_provider, Some(true));
    }

    #[test]
    fn client_capabilities_defaults() {
        let params: InitializeParams =
            serde_json::from_str(r#"{"capabilities":{}}"#).expect("解析初始化参数失败");
        assert!(!params.capabilities.dynamic_registration);
    }

    #[test]
    fn client_capabilities_parses_dynamic_registration() {
        let params: InitializeParams =
            serde_json::from_str(r#"{"capabilities":{"dynamicRegistration":true}}"#)
                .expect("解析初始化参数失败");
        assert!(params.capabilities.dynamic_registration);
    }

    #[test]
    fn initialize_result_serializes_with_camel_case() {
        let json = serde_json::to_value(initialize_result()).expect("序列化失败");
        assert_eq!(json["capabilities"]["textDocumentSync"], 2);
        assert!(json["capabilities"]["completionProvider"]["resolveProvider"].is_boolean());
        assert_eq!(json["serverInfo"]["name"], "ride-lsp");
    }
}
