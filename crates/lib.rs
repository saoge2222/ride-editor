//! Shared IPC protocol types for ride-editor ↔ ride-fm communication.
//! ride-editor 与 ride-fm 进程间通信的共享协议类型。

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileEntryData {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub depth: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TabData {
    pub name: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FullState {
    pub entries: Vec<FileEntryData>,
    pub current_path: String,
    pub tabs: Vec<TabData>,
    pub active_index: usize,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub id: u64,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub id: u64,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub state: Option<FullState>,
}
