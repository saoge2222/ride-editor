use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::rpc::message::ResponseError;
use crate::server::LspServer;

pub const CUSTOM_METHOD_PREFIX: &str = "$/";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolLayer {
    Base,
    TextSync,
    Language,
    Workspace,
    Window,
}

pub type CustomHandler = fn(&mut LspServer, &Value) -> Result<Value, ResponseError>;

pub struct ExtensionDescriptor {
    pub method: String,
    pub layer: ProtocolLayer,
    pub handler: CustomHandler,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Registration {
    pub id: String,
    pub method: String,
}

pub struct ExtensionRegistry {
    custom_handlers: HashMap<String, ExtensionDescriptor>,
    registered_capabilities: HashMap<String, Registration>,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        ExtensionRegistry {
            custom_handlers: HashMap::new(),
            registered_capabilities: HashMap::new(),
        }
    }

    pub fn register_custom(&mut self, descriptor: ExtensionDescriptor) -> Result<(), String> {
        if !descriptor.method.starts_with(CUSTOM_METHOD_PREFIX) {
            return Err("自定义方法必须以 $/ 开头".to_string());
        }
        if self.custom_handlers.contains_key(&descriptor.method) {
            return Err("方法重复注册".to_string());
        }
        self.custom_handlers
            .insert(descriptor.method.clone(), descriptor);
        Ok(())
    }

    pub fn is_custom(&self, method: &str) -> bool {
        self.custom_handlers.contains_key(method)
    }

    pub fn handler_for(&self, method: &str) -> Option<CustomHandler> {
        self.custom_handlers
            .get(method)
            .map(|descriptor| descriptor.handler)
    }

    pub fn record_registered(&mut self, registration: Registration) {
        self.registered_capabilities
            .insert(registration.id.clone(), registration);
    }

    pub fn remove_registered(&mut self, registration_id: &str) -> Result<(), String> {
        self.registered_capabilities
            .remove(registration_id)
            .map(|_| ())
            .ok_or_else(|| "注册项不存在".to_string())
    }

    pub fn registered_methods(&self) -> Vec<&str> {
        self.registered_capabilities
            .values()
            .map(|r| r.method.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noop(_server: &mut LspServer, _params: &Value) -> Result<Value, ResponseError> {
        Ok(Value::Null)
    }

    fn ping(_server: &mut LspServer, params: &Value) -> Result<Value, ResponseError> {
        Ok(serde_json::json!({"pong": params}))
    }

    fn make_registry() -> ExtensionRegistry {
        let mut registry = ExtensionRegistry::new();
        registry
            .register_custom(ExtensionDescriptor {
                method: "$/ping".to_string(),
                layer: ProtocolLayer::Base,
                handler: ping,
            })
            .expect("注册失败");
        registry
    }

    #[test]
    fn rejects_non_custom_prefix() {
        let mut registry = ExtensionRegistry::new();
        assert!(registry
            .register_custom(ExtensionDescriptor {
                method: "workspace/didChangeWatchedFiles".to_string(),
                layer: ProtocolLayer::Workspace,
                handler: noop,
            })
            .is_err());
    }

    #[test]
    fn rejects_duplicate_registration() {
        let mut registry = make_registry();
        assert!(registry
            .register_custom(ExtensionDescriptor {
                method: "$/ping".to_string(),
                layer: ProtocolLayer::Base,
                handler: noop,
            })
            .is_err());
    }

    #[test]
    fn dispatch_invokes_handler() {
        let registry = make_registry();
        let mut server = LspServer::new();
        let handler = registry.handler_for("$/ping").expect("应找到处理函数");
        let result = handler(&mut server, &serde_json::json!({"a": 1})).expect("分发失败");
        assert_eq!(result, serde_json::json!({"pong": {"a": 1}}));
    }

    #[test]
    fn dispatch_unknown_fails() {
        let registry = make_registry();
        assert!(registry.handler_for("$/nope").is_none());
    }

    #[test]
    fn registration_record_and_remove() {
        let mut registry = ExtensionRegistry::new();
        registry.record_registered(Registration {
            id: "watched".to_string(),
            method: "workspace/didChangeWatchedFiles".to_string(),
        });
        assert_eq!(
            registry.registered_methods(),
            vec!["workspace/didChangeWatchedFiles"]
        );
        assert!(registry.remove_registered("watched").is_ok());
        assert!(registry.remove_registered("watched").is_err());
    }
}
