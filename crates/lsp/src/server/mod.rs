pub mod capabilities;
pub mod handler;
pub mod lifecycle;
pub mod sync;

use std::io::{BufReader, BufWriter};

use serde_json::Value;

use crate::extensions::registry::{
    ExtensionDescriptor, ExtensionRegistry, ProtocolLayer,
};
use crate::features::completion::{CompletionBackend, DefaultCompletion};
use crate::features::definition::{DefaultDefinition, DefinitionBackend};
use crate::features::symbol_table::{DefaultSymbolTable, SymbolTable};
use crate::rpc::message::{Message, RequestId, ResponseError};
use crate::rpc::pending::PendingRegistry;
use crate::server::capabilities::{ClientCapabilities, ServerCapabilities};
use crate::server::lifecycle::ServerState;
use crate::server::sync::DocumentStore;
use crate::transport::frame;

pub struct LspServer {
    state: ServerState,
    client: ClientCapabilities,
    capabilities: ServerCapabilities,
    store: DocumentStore,
    pending: PendingRegistry,
    extensions: ExtensionRegistry,
    completion: Box<dyn CompletionBackend>,
    definition: Box<dyn DefinitionBackend>,
    symbols: Box<dyn SymbolTable>,
    saw_shutdown: bool,
    exit_code: i32,
}

fn extension_progress_noop(
    _server: &mut LspServer,
    _params: &Value,
) -> Result<Value, ResponseError> {
    Ok(Value::Null)
}

fn extension_ping(_server: &mut LspServer, params: &Value) -> Result<Value, ResponseError> {
    Ok(serde_json::json!({ "pong": params }))
}

impl LspServer {
    pub fn new() -> Self {
        let capabilities = capabilities::default_server_capabilities();
        let sync_kind = capabilities.text_document_sync;
        let mut extensions = ExtensionRegistry::new();
        let _ = extensions.register_custom(ExtensionDescriptor {
            method: "$/progress".to_string(),
            layer: ProtocolLayer::Base,
            handler: extension_progress_noop,
        });
        let _ = extensions.register_custom(ExtensionDescriptor {
            method: "$/ping".to_string(),
            layer: ProtocolLayer::Base,
            handler: extension_ping,
        });
        LspServer {
            state: ServerState::Uninitialized,
            client: ClientCapabilities::default(),
            capabilities,
            store: DocumentStore::new(sync_kind),
            pending: PendingRegistry::new(),
            extensions,
            completion: Box::new(DefaultCompletion),
            definition: Box::new(DefaultDefinition),
            symbols: Box::new(DefaultSymbolTable),
            saw_shutdown: false,
            exit_code: 1,
        }
    }

    pub(crate) fn cancel_pending(&mut self, id: &RequestId) -> bool {
        self.pending.cancel(id)
    }

    pub(crate) fn dynamic_registration_enabled(&self) -> bool {
        self.client.dynamic_registration
    }

    pub(crate) fn allocate_outgoing(&mut self, method: &str, params: Option<Value>) -> RequestId {
        self.pending.allocate(method, params)
    }

    pub(crate) fn take_outgoing(&mut self, id: &RequestId) -> Option<crate::rpc::pending::PendingEntry> {
        self.pending.take(id)
    }

    pub(crate) fn record_registration(
        &mut self,
        registration: crate::extensions::registry::Registration,
    ) {
        self.extensions.record_registered(registration);
    }

    pub(crate) fn dispatch_custom(
        &mut self,
        method: &str,
        params: &Value,
    ) -> Result<Value, ResponseError> {
        match self.extensions.handler_for(method) {
            Some(handler) => handler(self, params),
            None => Err(ResponseError {
                code: crate::rpc::message::ERROR_METHOD_NOT_FOUND,
                message: "未注册的自定义方法".to_string(),
            }),
        }
    }

    pub(crate) fn is_custom(&self, method: &str) -> bool {
        self.extensions.is_custom(method)
    }

    pub fn run(&mut self) -> i32 {
        let stdin = std::io::stdin();
        let mut reader = BufReader::new(stdin.lock());
        let stdout = std::io::stdout();
        let mut writer = BufWriter::new(stdout.lock());
        loop {
            let body = match frame::read_message(&mut reader) {
                Ok(Some(body)) => body,
                Ok(None) => break,
                Err(_) => break,
            };
            let mut outgoing: Vec<Message> = Vec::new();
            match Message::parse(&body) {
                Ok(msg) => handler::handle_message(self, msg, &mut outgoing),
                Err(e) => outgoing.push(Message::error_response(RequestId::Number(0), e)),
            }
            for msg in &outgoing {
                let json = serde_json::to_string(&msg.to_json()).expect("序列化 LSP 消息失败");
                frame::write_message(&mut writer, &json).expect("写入 LSP 消息失败");
            }
            if self.state == ServerState::Exited {
                return self.exit_code;
            }
        }
        self.exit_code
    }
}
