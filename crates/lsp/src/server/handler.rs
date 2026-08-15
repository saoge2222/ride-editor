use serde_json::Value;

use crate::document::{DocumentPayload, DocumentSource};
use crate::features::definition::identifier_at;
use crate::features::diagnostic::{BraceDiagnostic, DiagnosticBackend};
use crate::rpc::message::{
    Message, Method, Notification, Request, ResponseError, ERROR_INVALID_PARAMS,
    ERROR_METHOD_NOT_FOUND,
};
use crate::server::capabilities::{InitializeParams, ServerInfo};
use crate::server::lifecycle::ServerState;
use crate::server::LspServer;
use crate::types::completion::{CompletionParams, CompletionResponse};
use crate::types::diagnostic::{Diagnostic, PublishDiagnosticsParams};
use crate::types::position::TextDocumentPositionParams;
use crate::types::text_document::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams,
};

pub fn handle_message(server: &mut LspServer, msg: Message, outgoing: &mut Vec<Message>) {
    match msg {
        Message::Request(req) => handle_request(server, req, outgoing),
        Message::Notification(notif) => handle_notification(server, notif, outgoing),
        Message::Response(res) => handle_response(server, res),
    }
}

fn handle_request(server: &mut LspServer, req: Request, outgoing: &mut Vec<Message>) {
    if let Err(e) = server.state.validate_request(&req.method) {
        outgoing.push(Message::error_response(req.id, e));
        return;
    }
    let params = req.params.as_ref();
    let result = match Method::from_str(&req.method) {
        Some(Method::Initialize) => handle_initialize(server, params),
        Some(Method::Shutdown) => handle_shutdown(server, outgoing),
        Some(Method::Completion) => handle_completion(server, params),
        Some(Method::CompletionResolve) => handle_completion_resolve(params),
        Some(Method::Definition) => handle_definition(server, params),
        Some(Method::Hover) => handle_hover(server, params),
        Some(Method::References) => Ok(serde_json::json!([])),
        Some(Method::CodeAction) => Ok(serde_json::json!([])),
        Some(Method::DocumentSymbol) => handle_document_symbol(server, params),
        Some(_) => Err(ResponseError {
            code: ERROR_METHOD_NOT_FOUND,
            message: "该方法尚未实现".to_string(),
        }),
        None => {
            if server.is_custom(&req.method) {
                let empty = serde_json::json!({});
                server.dispatch_custom(&req.method, params.unwrap_or(&empty))
            } else {
                Err(ResponseError {
                    code: ERROR_METHOD_NOT_FOUND,
                    message: "未知方法".to_string(),
                })
            }
        }
    };
    match result {
        Ok(value) => outgoing.push(Message::result_response(req.id, value)),
        Err(e) => outgoing.push(Message::error_response(req.id, e)),
    }
}

fn handle_notification(server: &mut LspServer, notif: Notification, outgoing: &mut Vec<Message>) {
    if let Err(_) = server.state.validate_notification(&notif.method) {
        return;
    }
    match Method::from_str(&notif.method) {
        Some(Method::Initialized) => {
            let _ = server.state.transition(ServerState::Initialized);
            if server.dynamic_registration_enabled() {
                register_watched_files(server, outgoing);
            }
        }
        Some(Method::Exit) => {
            server.exit_code = if server.saw_shutdown { 0 } else { 1 };
            let _ = server.state.transition(ServerState::Exited);
        }
        Some(Method::DidOpen) => handle_did_open(server, notif.params.as_ref(), outgoing),
        Some(Method::DidChange) => handle_did_change(server, notif.params.as_ref(), outgoing),
        Some(Method::DidClose) => handle_did_close(server, notif.params.as_ref()),
        Some(Method::DidSave) => handle_did_save(server, notif.params.as_ref()),
        Some(Method::CancelRequest) => handle_cancel_request(server, notif.params.as_ref()),
        Some(_) => {}
        None => {
            let params = notif.params.unwrap_or(serde_json::json!({}));
            let _ = server.dispatch_custom(&notif.method, &params);
        }
    }
}

fn handle_response(server: &mut LspServer, res: crate::rpc::message::Response) {
    if let Some(entry) = server.take_outgoing(&res.id) {
        if entry.method == "client/registerCapability" {
            record_register_result(server, &entry.params);
        }
    }
}

fn register_watched_files(server: &mut LspServer, outgoing: &mut Vec<Message>) {
    let params = serde_json::json!({
        "registrations": [{
            "id": "watched-files",
            "method": "workspace/didChangeWatchedFiles"
        }]
    });
    let id = server.allocate_outgoing("client/registerCapability", Some(params.clone()));
    outgoing.push(Message::request(
        id,
        "client/registerCapability",
        Some(params),
    ));
}

fn record_register_result(server: &mut LspServer, params: &Option<serde_json::Value>) {
    let registrations = params
        .as_ref()
        .and_then(|p| p.get("registrations"))
        .and_then(|r| r.as_array());
    let Some(registrations) = registrations else {
        return;
    };
    let parsed: Vec<crate::extensions::registry::Registration> =
        serde_json::from_value(serde_json::Value::Array(registrations.clone()))
            .unwrap_or_default();
    for registration in parsed {
        server.record_registration(registration);
    }
}

fn handle_cancel_request(server: &mut LspServer, params: Option<&Value>) {
    let Some(value) = params else {
        return;
    };
    let id: crate::rpc::message::RequestId = match serde_json::from_value(
        value.get("id").cloned().unwrap_or(Value::Null),
    ) {
        Ok(id) => id,
        Err(_) => return,
    };
    let _ = server.cancel_pending(&id);
}

fn handle_initialize(server: &mut LspServer, params: Option<&Value>) -> Result<Value, ResponseError> {
    let params: InitializeParams = match params {
        Some(value) => serde_json::from_value(value.clone()).map_err(|_| ResponseError {
            code: ERROR_INVALID_PARAMS,
            message: "initialize 参数无效".to_string(),
        })?,
        None => InitializeParams::default(),
    };
    server.client = params.capabilities;
    server
        .state
        .transition(ServerState::Initializing)
        .map_err(|e| e)?;
    let result = crate::server::capabilities::InitializeResult {
        capabilities: server.capabilities.clone(),
        server_info: ServerInfo {
            name: crate::server::capabilities::SERVER_NAME.to_string(),
            version: crate::server::capabilities::SERVER_VERSION.to_string(),
        },
    };
    serde_json::to_value(result).map_err(|_| ResponseError {
        code: ERROR_INVALID_PARAMS,
        message: "构造初始化结果失败".to_string(),
    })
}

fn handle_shutdown(server: &mut LspServer, outgoing: &mut Vec<Message>) -> Result<Value, ResponseError> {
    server.saw_shutdown = true;
    server.state.transition(ServerState::Shutdown)?;
    outgoing.push(Message::notification("$/cleanup", Some(serde_json::json!({}))));
    Ok(Value::Null)
}

fn handle_did_open(server: &mut LspServer, params: Option<&Value>, outgoing: &mut Vec<Message>) {
    let Some(value) = params else {
        return;
    };
    let params: DidOpenTextDocumentParams = match serde_json::from_value(value.clone()) {
        Ok(params) => params,
        Err(_) => return,
    };
    let payload = DocumentPayload::from_text_document_item(&params.text_document);
    if server.store.open(payload).is_ok() {
        push_diagnostics(server, &params.text_document.uri, outgoing);
    }
}

fn handle_did_change(server: &mut LspServer, params: Option<&Value>, outgoing: &mut Vec<Message>) {
    let Some(value) = params else {
        return;
    };
    let params: DidChangeTextDocumentParams = match serde_json::from_value(value.clone()) {
        Ok(params) => params,
        Err(_) => return,
    };
    let uri = params.text_document.uri.clone();
    if server
        .store
        .change(&uri, params.text_document.version, &params.content_changes)
        .is_ok()
    {
        push_diagnostics(server, &uri, outgoing);
    }
}

fn handle_did_close(server: &mut LspServer, params: Option<&Value>) {
    let Some(value) = params else {
        return;
    };
    let params: DidCloseTextDocumentParams = match serde_json::from_value(value.clone()) {
        Ok(params) => params,
        Err(_) => return,
    };
    let _ = server.store.close(&params.text_document.uri);
}

fn handle_did_save(server: &mut LspServer, params: Option<&Value>) {
    let Some(value) = params else {
        return;
    };
    let params: DidSaveTextDocumentParams = match serde_json::from_value(value.clone()) {
        Ok(params) => params,
        Err(_) => return,
    };
    let _ = server.store.save(&params.text_document.uri);
}

fn push_diagnostics(server: &mut LspServer, uri: &str, outgoing: &mut Vec<Message>) {
    let diagnostics: Vec<Diagnostic> = match server.store.get(uri) {
        Some(document) => {
            let mut backend = BraceDiagnostic;
            backend.scan(document)
        }
        None => Vec::new(),
    };
    let params = PublishDiagnosticsParams {
        uri: uri.to_string(),
        diagnostics,
    };
    let value = serde_json::to_value(params).expect("序列化诊断失败");
    outgoing.push(Message::notification(
        "textDocument/publishDiagnostics",
        Some(value),
    ));
}

fn handle_completion(server: &mut LspServer, params: Option<&Value>) -> Result<Value, ResponseError> {
    let params: CompletionParams = parse_params(params)?;
    let document = server
        .store
        .get(&params.text_document.uri)
        .ok_or_else(|| ResponseError {
            code: crate::rpc::message::ERROR_REQUEST_FAILED,
            message: "文档未打开".to_string(),
        })?;
    let trigger = params
        .context
        .as_ref()
        .and_then(|ctx| ctx.trigger_character.as_deref());
    let items = server.completion.complete(document, &params.position, trigger);
    let response = CompletionResponse::Items(items);
    serde_json::to_value(response).map_err(|_| ResponseError {
        code: crate::rpc::message::ERROR_INTERNAL,
        message: "序列化补全结果失败".to_string(),
    })
}

fn handle_completion_resolve(params: Option<&Value>) -> Result<Value, ResponseError> {
    let item = parse_params::<crate::types::completion::CompletionItem>(params)?;
    let resolved = crate::types::completion::CompletionItem {
        detail: Some(item.label.clone()),
        ..item
    };
    serde_json::to_value(resolved).map_err(|_| ResponseError {
        code: crate::rpc::message::ERROR_INTERNAL,
        message: "序列化补全项失败".to_string(),
    })
}

fn handle_definition(server: &mut LspServer, params: Option<&Value>) -> Result<Value, ResponseError> {
    let params: TextDocumentPositionParams = parse_params(params)?;
    let document = server
        .store
        .get(&params.text_document.uri)
        .ok_or_else(|| ResponseError {
            code: crate::rpc::message::ERROR_REQUEST_FAILED,
            message: "文档未打开".to_string(),
        })?;
    let location = server.definition.define(document, &params.position);
    match location {
        Some(location) => serde_json::to_value(location).map_err(|_| ResponseError {
            code: crate::rpc::message::ERROR_INTERNAL,
            message: "序列化定位结果失败".to_string(),
        }),
        None => Ok(Value::Null),
    }
}

fn handle_hover(server: &mut LspServer, params: Option<&Value>) -> Result<Value, ResponseError> {
    let params: TextDocumentPositionParams = parse_params(params)?;
    let document = server
        .store
        .get(&params.text_document.uri)
        .ok_or_else(|| ResponseError {
            code: crate::rpc::message::ERROR_REQUEST_FAILED,
            message: "文档未打开".to_string(),
        })?;
    match identifier_at(document, &params.position) {
        Some(name) => Ok(serde_json::json!({
            "contents": { "kind": "plaintext", "value": name }
        })),
        None => Ok(Value::Null),
    }
}

fn handle_document_symbol(server: &mut LspServer, params: Option<&Value>) -> Result<Value, ResponseError> {
    let params: TextDocumentPositionParams = parse_params(params)?;
    let document = server
        .store
        .get(&params.text_document.uri)
        .ok_or_else(|| ResponseError {
            code: crate::rpc::message::ERROR_REQUEST_FAILED,
            message: "文档未打开".to_string(),
        })?;
    let symbols = server.symbols.build(document);
    serde_json::to_value(symbols).map_err(|_| ResponseError {
        code: crate::rpc::message::ERROR_INTERNAL,
        message: "序列化符号表失败".to_string(),
    })
}

fn parse_params<T: serde::de::DeserializeOwned>(params: Option<&Value>) -> Result<T, ResponseError> {
    let value = params.ok_or_else(|| ResponseError {
        code: ERROR_INVALID_PARAMS,
        message: "缺少参数".to_string(),
    })?;
    serde_json::from_value(value.clone()).map_err(|_| ResponseError {
        code: ERROR_INVALID_PARAMS,
        message: "参数无效".to_string(),
    })
}
