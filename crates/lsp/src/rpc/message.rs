use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const PROTOCOL_VERSION: &str = "2.0";

pub const ERROR_PARSE: i64 = -32700;
pub const ERROR_INVALID_REQUEST: i64 = -32600;
pub const ERROR_METHOD_NOT_FOUND: i64 = -32601;
pub const ERROR_INVALID_PARAMS: i64 = -32602;
pub const ERROR_INTERNAL: i64 = -32603;
pub const ERROR_SERVER_NOT_INITIALIZED: i64 = -32002;
pub const ERROR_REQUEST_FAILED: i64 = -32803;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
    Number(u64),
    String(String),
}

impl RequestId {
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            RequestId::Number(n) => Some(*n),
            RequestId::String(_) => None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Request {
    pub jsonrpc: String,
    pub id: RequestId,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub params: Option<Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ResponseError {
    pub code: i64,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Response {
    pub jsonrpc: String,
    pub id: RequestId,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub error: Option<ResponseError>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Notification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub params: Option<Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    Request(Request),
    Response(Response),
    Notification(Notification),
}

impl Message {
    pub fn from_json(v: Value) -> Result<Message, ResponseError> {
        let jsonrpc_ok = v
            .get("jsonrpc")
            .and_then(|j| j.as_str())
            .map(|s| s == PROTOCOL_VERSION)
            .unwrap_or(false);
        if !jsonrpc_ok {
            return Err(ResponseError {
                code: ERROR_INVALID_REQUEST,
                message: "invalid jsonrpc version".to_string(),
            });
        }
        let has_method = v.get("method").is_some();
        let has_id = v.get("id").is_some();
        if has_method {
            let method = v
                .get("method")
                .and_then(|m| m.as_str())
                .ok_or_else(|| ResponseError {
                    code: ERROR_INVALID_REQUEST,
                    message: "method must be a string".to_string(),
                })?
                .to_string();
            if has_id {
                let id: RequestId = serde_json::from_value(v.get("id").cloned().unwrap_or(Value::Null))
                    .map_err(|_| ResponseError {
                        code: ERROR_INVALID_REQUEST,
                        message: "invalid request id".to_string(),
                    })?;
                Ok(Message::Request(Request {
                    jsonrpc: PROTOCOL_VERSION.to_string(),
                    id,
                    method,
                    params: v.get("params").cloned(),
                }))
            } else {
                Ok(Message::Notification(Notification {
                    jsonrpc: PROTOCOL_VERSION.to_string(),
                    method,
                    params: v.get("params").cloned(),
                }))
            }
        } else if has_id {
            let id: RequestId = serde_json::from_value(v.get("id").cloned().unwrap_or(Value::Null))
                .map_err(|_| ResponseError {
                    code: ERROR_INVALID_REQUEST,
                    message: "invalid response id".to_string(),
                })?;
            Ok(Message::Response(Response {
                jsonrpc: PROTOCOL_VERSION.to_string(),
                id,
                result: v.get("result").cloned(),
                error: serde_json::from_value(v.get("error").cloned().unwrap_or(Value::Null)).ok(),
            }))
        } else {
            Err(ResponseError {
                code: ERROR_INVALID_REQUEST,
                message: "message is neither request nor response".to_string(),
            })
        }
    }

    pub fn to_json(&self) -> Value {
        match self {
            Message::Request(req) => {
                let mut obj = serde_json::Map::new();
                obj.insert("jsonrpc".to_string(), Value::String(req.jsonrpc.clone()));
                obj.insert(
                    "id".to_string(),
                    serde_json::to_value(&req.id).expect("序列化请求 id 失败"),
                );
                obj.insert("method".to_string(), Value::String(req.method.clone()));
                if let Some(params) = &req.params {
                    obj.insert("params".to_string(), params.clone());
                }
                Value::Object(obj)
            }
            Message::Response(res) => {
                let mut obj = serde_json::Map::new();
                obj.insert("jsonrpc".to_string(), Value::String(res.jsonrpc.clone()));
                obj.insert(
                    "id".to_string(),
                    serde_json::to_value(&res.id).expect("序列化响应 id 失败"),
                );
                if let Some(result) = &res.result {
                    obj.insert("result".to_string(), result.clone());
                }
                if let Some(error) = &res.error {
                    obj.insert(
                        "error".to_string(),
                        serde_json::to_value(error).expect("序列化错误信息失败"),
                    );
                }
                Value::Object(obj)
            }
            Message::Notification(notif) => {
                let mut obj = serde_json::Map::new();
                obj.insert("jsonrpc".to_string(), Value::String(notif.jsonrpc.clone()));
                obj.insert("method".to_string(), Value::String(notif.method.clone()));
                if let Some(params) = &notif.params {
                    obj.insert("params".to_string(), params.clone());
                }
                Value::Object(obj)
            }
        }
    }

    pub fn parse(body: &str) -> Result<Message, ResponseError> {
        let value: Value = serde_json::from_str(body).map_err(|_| ResponseError {
            code: ERROR_PARSE,
            message: "parse error".to_string(),
        })?;
        Message::from_json(value)
    }

    pub fn request(id: RequestId, method: &str, params: Option<Value>) -> Message {
        Message::Request(Request {
            jsonrpc: PROTOCOL_VERSION.to_string(),
            id,
            method: method.to_string(),
            params,
        })
    }

    pub fn notification(method: &str, params: Option<Value>) -> Message {
        Message::Notification(Notification {
            jsonrpc: PROTOCOL_VERSION.to_string(),
            method: method.to_string(),
            params,
        })
    }

    pub fn result_response(id: RequestId, result: Value) -> Message {
        Message::Response(Response {
            jsonrpc: PROTOCOL_VERSION.to_string(),
            id,
            result: Some(result),
            error: None,
        })
    }

    pub fn error_response(id: RequestId, error: ResponseError) -> Message {
        Message::Response(Response {
            jsonrpc: PROTOCOL_VERSION.to_string(),
            id,
            result: None,
            error: Some(error),
        })
    }

    pub fn is_request(&self) -> bool {
        matches!(self, Message::Request(_))
    }

    pub fn is_response(&self) -> bool {
        matches!(self, Message::Response(_))
    }

    pub fn is_notification(&self) -> bool {
        matches!(self, Message::Notification(_))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Initialize,
    Initialized,
    Shutdown,
    Exit,
    DidOpen,
    DidChange,
    DidClose,
    DidSave,
    Completion,
    CompletionResolve,
    Definition,
    Hover,
    References,
    CodeAction,
    DocumentSymbol,
    RegisterCapability,
    UnregisterCapability,
    CancelRequest,
    Custom,
}

impl Method {
    pub fn from_str(name: &str) -> Option<Method> {
        match name {
            "initialize" => Some(Method::Initialize),
            "initialized" => Some(Method::Initialized),
            "shutdown" => Some(Method::Shutdown),
            "exit" => Some(Method::Exit),
            "textDocument/didOpen" => Some(Method::DidOpen),
            "textDocument/didChange" => Some(Method::DidChange),
            "textDocument/didClose" => Some(Method::DidClose),
            "textDocument/didSave" => Some(Method::DidSave),
            "textDocument/completion" => Some(Method::Completion),
            "completionItem/resolve" => Some(Method::CompletionResolve),
            "textDocument/definition" => Some(Method::Definition),
            "textDocument/hover" => Some(Method::Hover),
            "textDocument/references" => Some(Method::References),
            "textDocument/codeAction" => Some(Method::CodeAction),
            "textDocument/documentSymbol" => Some(Method::DocumentSymbol),
            "client/registerCapability" => Some(Method::RegisterCapability),
            "client/unregisterCapability" => Some(Method::UnregisterCapability),
            "$/cancelRequest" => Some(Method::CancelRequest),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Method::Initialize => "initialize",
            Method::Initialized => "initialized",
            Method::Shutdown => "shutdown",
            Method::Exit => "exit",
            Method::DidOpen => "textDocument/didOpen",
            Method::DidChange => "textDocument/didChange",
            Method::DidClose => "textDocument/didClose",
            Method::DidSave => "textDocument/didSave",
            Method::Completion => "textDocument/completion",
            Method::CompletionResolve => "completionItem/resolve",
            Method::Definition => "textDocument/definition",
            Method::Hover => "textDocument/hover",
            Method::References => "textDocument/references",
            Method::CodeAction => "textDocument/codeAction",
            Method::DocumentSymbol => "textDocument/documentSymbol",
            Method::RegisterCapability => "client/registerCapability",
            Method::UnregisterCapability => "client/unregisterCapability",
            Method::CancelRequest => "$/cancelRequest",
            Method::Custom => "$/custom",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_roundtrip() {
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#;
        let msg = Message::parse(body).expect("解析请求失败");
        assert!(msg.is_request());
        let json = msg.to_json();
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 1);
        assert_eq!(json["method"], "initialize");
        assert!(json.get("params").is_some());
    }

    #[test]
    fn notification_has_no_id() {
        let body = r#"{"jsonrpc":"2.0","method":"initialized"}"#;
        let msg = Message::parse(body).expect("解析通知失败");
        assert!(msg.is_notification());
        let json = msg.to_json();
        assert!(json.get("id").is_none());
        assert_eq!(json["method"], "initialized");
    }

    #[test]
    fn response_roundtrip() {
        let msg = Message::result_response(RequestId::Number(7), serde_json::json!({"ok": true}));
        let json = msg.to_json();
        assert_eq!(json["id"], 7);
        assert_eq!(json["result"]["ok"], true);
        assert!(json.get("error").is_none());
    }

    #[test]
    fn error_response_shape() {
        let err = ResponseError {
            code: ERROR_METHOD_NOT_FOUND,
            message: "method not found".to_string(),
        };
        let msg = Message::error_response(RequestId::Number(2), err);
        let json = msg.to_json();
        assert_eq!(json["error"]["code"], -32601);
        assert!(json.get("result").is_none());
    }

    #[test]
    fn parse_bad_json_gives_minus_32700() {
        let err = Message::parse("not json").expect_err("应解析失败");
        assert_eq!(err.code, ERROR_PARSE);
    }

    #[test]
    fn parse_wrong_version_gives_invalid_request() {
        let err = Message::parse(r#"{"jsonrpc":"1.0","id":1,"method":"initialize"}"#)
            .expect_err("应拒绝错误版本");
        assert_eq!(err.code, ERROR_INVALID_REQUEST);
    }

    #[test]
    fn parse_neither_gives_invalid_request() {
        let err = Message::parse(r#"{"jsonrpc":"2.0","foo":"bar"}"#).expect_err("应拒绝");
        assert_eq!(err.code, ERROR_INVALID_REQUEST);
    }

    #[test]
    fn string_request_id_roundtrip() {
        let msg = Message::request(
            RequestId::String("abc".to_string()),
            "shutdown",
            None,
        );
        let json = msg.to_json();
        assert_eq!(json["id"], "abc");
        assert!(msg.is_request());
    }

    #[test]
    fn method_mapping() {
        assert_eq!(Method::from_str("initialize"), Some(Method::Initialize));
        assert_eq!(Method::from_str("textDocument/didChange"), Some(Method::DidChange));
        assert_eq!(Method::from_str("unknown/method"), None);
        assert_eq!(Method::as_str(&Method::DidOpen), "textDocument/didOpen");
    }
}
