use std::collections::HashMap;

use serde_json::Value;

use crate::rpc::message::RequestId;

pub const OUTGOING_ID_BASE: u64 = 1_000_000;

#[derive(Clone, Debug, PartialEq)]
pub struct PendingEntry {
    pub id: RequestId,
    pub method: String,
    pub params: Option<Value>,
}

pub struct PendingRegistry {
    next_outgoing_id: u64,
    entries: HashMap<RequestId, PendingEntry>,
}

impl PendingRegistry {
    pub fn new() -> Self {
        PendingRegistry {
            next_outgoing_id: OUTGOING_ID_BASE,
            entries: HashMap::new(),
        }
    }

    pub fn allocate(&mut self, method: &str, params: Option<Value>) -> RequestId {
        let id = RequestId::Number(self.next_outgoing_id);
        self.next_outgoing_id += 1;
        self.entries.insert(
            id.clone(),
            PendingEntry {
                id: id.clone(),
                method: method.to_string(),
                params,
            },
        );
        id
    }

    pub fn take(&mut self, id: &RequestId) -> Option<PendingEntry> {
        self.entries.remove(id)
    }

    pub fn cancel(&mut self, id: &RequestId) -> bool {
        self.entries.remove(id).is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_starts_at_base() {
        let mut registry = PendingRegistry::new();
        let id = registry.allocate("client/registerCapability", None);
        assert_eq!(id, RequestId::Number(OUTGOING_ID_BASE));
        assert!(!registry.is_empty());
    }

    #[test]
    fn allocate_increments() {
        let mut registry = PendingRegistry::new();
        let first = registry.allocate("a", None);
        let second = registry.allocate("b", None);
        assert_eq!(first.as_u64(), Some(OUTGOING_ID_BASE));
        assert_eq!(second.as_u64(), Some(OUTGOING_ID_BASE + 1));
    }

    #[test]
    fn allocate_keeps_params() {
        let mut registry = PendingRegistry::new();
        let params = serde_json::json!({"registrations": []});
        let id = registry.allocate("client/registerCapability", Some(params.clone()));
        let entry = registry.take(&id).expect("应取到挂起条目");
        assert_eq!(entry.params, Some(params));
    }

    #[test]
    fn take_removes_entry() {
        let mut registry = PendingRegistry::new();
        let id = registry.allocate("client/registerCapability", None);
        let entry = registry.take(&id).expect("应取到挂起条目");
        assert_eq!(entry.method, "client/registerCapability");
        assert!(registry.take(&id).is_none());
        assert!(registry.is_empty());
    }

    #[test]
    fn cancel_removes_entry() {
        let mut registry = PendingRegistry::new();
        let id = registry.allocate("x", None);
        assert!(registry.cancel(&id));
        assert!(!registry.cancel(&id));
        assert!(registry.is_empty());
    }

    #[test]
    fn unknown_id_take_is_none() {
        let mut registry = PendingRegistry::new();
        assert!(registry.take(&RequestId::Number(1)).is_none());
    }
}
