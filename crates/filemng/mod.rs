//! File management IPC client — spawns ride-fm daemon and communicates via JSON.
//! 文件管理 IPC 客户端 — 启动 ride-fm 守护进程并通过 JSON 通信。
//!
//! Provides the same synchronous API as the old `FileManager` while
//! delegating all file-system operations to a separate process.

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use ride_editor::{FullState, Request, Response};

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub depth: usize,
}

pub struct OpenFile {
    pub name: String,
    pub path: String,
    pub content: String,
}

pub struct FileMngClient {
    stdin: BufWriter<std::process::ChildStdin>,
    stdout: BufReader<std::process::ChildStdout>,
    child: Child,
    next_id: u64,
    entries: Vec<FileEntry>,
    current_dir: String,
    open_files: Vec<OpenFile>,
    active_file_index: usize,
    active_content_cache: String,
}

impl FileMngClient {
    pub fn spawn() -> Self {
        let fm_path = fm_binary_path();

        let mut child = Command::new(&fm_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| panic!("Failed to spawn ride-fm at {}: {e}", fm_path.display()));

        let stdin = BufWriter::new(child.stdin.take().expect("stdin pipe"));
        let stdout = BufReader::new(child.stdout.take().expect("stdout pipe"));

        let mut client = Self {
            stdin,
            stdout,
            child,
            next_id: 0,
            entries: Vec::new(),
            current_dir: String::new(),
            open_files: Vec::new(),
            active_file_index: 0,
            active_content_cache: String::new(),
        };
        client.next_id = 1;
        let state = client.send("get_state", serde_json::json!({}));
        client.apply_state(state);
        client
    }

    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn send(&mut self, method: &str, params: serde_json::Value) -> FullState {
        let id = self.next_id();
        let req = Request { id, method: method.to_string(), params };
        let line = serde_json::to_string(&req).unwrap() + "\n";
        self.stdin.write_all(line.as_bytes()).expect("write to fm stdin");
        self.stdin.flush().expect("flush fm stdin");

        let mut response = String::new();
        self.stdout.read_line(&mut response).expect("read from fm stdout");
        let resp: Response = serde_json::from_str(&response).expect("parse fm response");

        if !resp.ok {
            panic!("fm error: {}", resp.error.as_deref().unwrap_or("unknown"));
        }
        resp.state.expect("fm response missing state")
    }

    fn apply_state(&mut self, state: FullState) {
        self.entries = state
            .entries
            .iter()
            .map(|e| FileEntry {
                name: e.name.clone(),
                path: e.path.clone(),
                is_dir: e.is_dir,
                depth: e.depth,
            })
            .collect();
        self.current_dir = state.current_path;
        self.open_files = state
            .tabs
            .iter()
            .enumerate()
            .map(|(i, t)| OpenFile {
                name: t.name.clone(),
                path: t.path.clone(),
                content: if i == state.active_index { state.content.clone() } else { String::new() },
            })
            .collect();
        self.active_file_index = state.active_index;
        self.active_content_cache = state.content;
    }

    pub fn entries(&self) -> &[FileEntry] {
        &self.entries
    }

    pub fn current_dir(&self) -> &str {
        &self.current_dir
    }

    pub fn toggle_expand(&mut self, path: &str) {
        let state = self.send("toggle_expand", serde_json::json!({ "path": path }));
        self.apply_state(state);
    }

    pub fn open_file(&mut self, path: &str) -> Option<usize> {
        let state = self.send("open_file", serde_json::json!({ "path": path }));
        let idx = state.active_index;
        self.apply_state(state);
        Some(idx)
    }

    pub fn close_file(&mut self, index: usize) {
        let state = self.send("close_file", serde_json::json!({ "index": index }));
        self.apply_state(state);
    }

    pub fn open_files(&self) -> &[OpenFile] {
        &self.open_files
    }

    pub fn active_file_index(&self) -> usize {
        self.active_file_index
    }

    pub fn set_active_file(&mut self, index: usize) {
        let state = self.send("set_active_file", serde_json::json!({ "index": index }));
        self.apply_state(state);
    }

    pub fn active_content(&self) -> &str {
        &self.active_content_cache
    }

    pub fn update_active_content(&mut self, content: &str) {
        let state = self.send("update_content", serde_json::json!({ "text": content }));
        self.apply_state(state);
    }

    pub fn save_file(&mut self) -> std::io::Result<()> {
        let state = self.send("save_file", serde_json::json!({}));
        self.apply_state(state);
        Ok(())
    }
}

impl Drop for FileMngClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

fn fm_binary_path() -> PathBuf {
    if let Ok(path) = std::env::var("RIDE_FM_PATH") {
        return PathBuf::from(path);
    }
    let mut exe = std::env::current_exe().expect("Cannot get exe path");
    exe.set_file_name("ride-fm");
    if cfg!(windows) {
        exe.set_extension("exe");
    }
    exe
}
