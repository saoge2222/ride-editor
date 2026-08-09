//! File manager daemon — standalone process for directory browsing and file I/O.
//! 文件管理守护进程 — 独立的目录浏览与文件 IO 进程。
//!
//! Communicates with the editor via newline-delimited JSON over stdin/stdout.
//! 通过 stdin/stdout 的换行分隔 JSON 与编辑器通信。

use std::collections::HashSet;
use std::fs;
use std::io::{self, BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};

use ride_editor::{FileEntryData, FullState, Request, Response, TabData};

#[derive(Debug, Clone)]
struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    depth: usize,
}

struct OpenFile {
    name: String,
    path: String,
    content: String,
}

struct FileManager {
    current_dir: PathBuf,
    entries: Vec<FileEntry>,
    open_files: Vec<OpenFile>,
    active_file_index: usize,
    expanded_dirs: HashSet<String>,
}

impl FileManager {
    fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let entries = Self::build_tree_entries(&current_dir, &HashSet::new());
        Self { current_dir, entries, open_files: Vec::new(), active_file_index: 0, expanded_dirs: HashSet::new() }
    }

    fn read_dir_entries(path: &Path) -> Vec<(String, String, bool)> {
        let mut result = Vec::new();
        if let Ok(dir) = fs::read_dir(path) {
            for entry in dir.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let path = entry.path().to_string_lossy().to_string();
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                result.push((name, path, is_dir));
            }
        }
        result.sort_by(|a, b| match (a.2, b.2) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.0.to_lowercase().cmp(&b.0.to_lowercase()),
        });
        result
    }

    fn build_tree_entries(root: &Path, expanded: &HashSet<String>) -> Vec<FileEntry> {
        let mut entries = Vec::new();
        Self::scan_tree(root, 0, expanded, &mut entries);
        entries
    }

    fn scan_tree(dir: &Path, depth: usize, expanded: &HashSet<String>, out: &mut Vec<FileEntry>) {
        let items = Self::read_dir_entries(dir);
        for (name, path, is_dir) in items {
            let path_buf = PathBuf::from(&path);
            out.push(FileEntry { name, path: path.clone(), is_dir, depth });
            if is_dir && expanded.contains(&path) {
                Self::scan_tree(&path_buf, depth + 1, expanded, out);
            }
        }
    }

    fn toggle_expand(&mut self, path: &str) {
        if self.expanded_dirs.contains(path) {
            self.expanded_dirs.remove(path);
        } else {
            self.expanded_dirs.insert(path.to_string());
        }
        self.entries = Self::build_tree_entries(&self.current_dir, &self.expanded_dirs);
    }

    fn open_file(&mut self, path: &str) -> Option<usize> {
        let p = Path::new(path);
        if p.is_file() {
            let name = p.file_name()?.to_string_lossy().to_string();
            let content = fs::read_to_string(p).unwrap_or_default();
            if let Some(idx) = self.open_files.iter().position(|f| f.path == path) {
                self.active_file_index = idx;
                return Some(idx);
            }
            self.open_files.push(OpenFile { name, path: path.to_string(), content });
            let idx = self.open_files.len() - 1;
            self.active_file_index = idx;
            Some(idx)
        } else {
            None
        }
    }

    fn close_file(&mut self, index: usize) {
        if index < self.open_files.len() {
            self.open_files.remove(index);
            if self.open_files.is_empty() {
                self.active_file_index = 0;
            } else if self.active_file_index >= self.open_files.len() {
                self.active_file_index = self.open_files.len() - 1;
            }
        }
    }

    fn set_active_file(&mut self, index: usize) {
        if index < self.open_files.len() {
            self.active_file_index = index;
        }
    }

    fn active_content(&self) -> &str {
        self.open_files.get(self.active_file_index).map(|f| f.content.as_str()).unwrap_or("")
    }

    fn update_active_content(&mut self, content: &str) {
        if let Some(file) = self.open_files.get_mut(self.active_file_index) {
            file.content = content.to_string();
        }
    }

    fn save_file(&mut self) -> std::io::Result<()> {
        if self.open_files.is_empty() {
            return Ok(());
        }
        let file = &self.open_files[self.active_file_index];
        fs::write(&file.path, &file.content)
    }

    fn full_state(&self) -> FullState {
        FullState {
            entries: self
                .entries
                .iter()
                .map(|e| FileEntryData { name: e.name.clone(), path: e.path.clone(), is_dir: e.is_dir, depth: e.depth })
                .collect(),
            current_path: self.current_dir.to_string_lossy().to_string(),
            tabs: self
                .open_files
                .iter()
                .map(|f| TabData { name: f.name.clone(), path: f.path.clone() })
                .collect(),
            active_index: self.active_file_index,
            content: self.active_content().to_string(),
        }
    }

    fn build_response(&self, req: &Request, ok: bool, msg: Option<&str>) -> Response {
        Response {
            id: req.id,
            ok,
            error: msg.map(|s| s.to_string()),
            state: if ok { Some(self.full_state()) } else { None },
        }
    }
}

fn main() {
    let mut fm = FileManager::new();
    let stdin = io::stdin().lock();
    let mut stdout = BufWriter::new(io::stdout());

    for line in stdin.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }

        let req: Request = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = Response {
                    id: 0,
                    ok: false,
                    error: Some(format!("parse error: {e}")),
                    state: None,
                };
                writeln!(stdout, "{}", serde_json::to_string(&resp).unwrap()).ok();
                stdout.flush().ok();
                continue;
            }
        };

        let resp = match req.method.as_str() {
            "get_state" => fm.build_response(&req, true, None),
            "toggle_expand" => {
                if let Some(path) = req.params.get("path").and_then(|v| v.as_str()) {
                    fm.toggle_expand(path);
                    fm.build_response(&req, true, None)
                } else {
                    fm.build_response(&req, false, Some("missing path"))
                }
            }
            "open_file" => {
                if let Some(path) = req.params.get("path").and_then(|v| v.as_str()) {
                    fm.open_file(path);
                    fm.build_response(&req, true, None)
                } else {
                    fm.build_response(&req, false, Some("missing path"))
                }
            }
            "close_file" => {
                if let Some(index) = req.params.get("index").and_then(|v| v.as_u64()) {
                    fm.close_file(index as usize);
                    fm.build_response(&req, true, None)
                } else {
                    fm.build_response(&req, false, Some("missing index"))
                }
            }
            "set_active_file" => {
                if let Some(index) = req.params.get("index").and_then(|v| v.as_u64()) {
                    fm.set_active_file(index as usize);
                    fm.build_response(&req, true, None)
                } else {
                    fm.build_response(&req, false, Some("missing index"))
                }
            }
            "save_file" => {
                let ok = fm.save_file().is_ok();
                fm.build_response(&req, ok, if !ok { Some("save failed") } else { None })
            }
            "update_content" => {
                if let Some(text) = req.params.get("text").and_then(|v| v.as_str()) {
                    fm.update_active_content(text);
                    fm.build_response(&req, true, None)
                } else {
                    fm.build_response(&req, false, Some("missing text"))
                }
            }
            _ => fm.build_response(&req, false, Some(&format!("unknown method: {}", req.method))),
        };

        writeln!(stdout, "{}", serde_json::to_string(&resp).unwrap()).ok();
        stdout.flush().ok();
    }
}
