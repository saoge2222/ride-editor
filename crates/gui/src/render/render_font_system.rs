use std::fs;
use std::path::{Path, PathBuf};

use super::render_font::Font;

const ENV_FONT_FAMILY: &str = "RIDE_FONT_FAMILY";
const FONT_EXTENSIONS: [&str; 2] = ["ttf", "otf"];
const SYSTEM_FONT_DIRS: [&str; 2] = ["/usr/share/fonts", "/usr/local/share/fonts"];
const USER_FONT_SUBDIRS: [&str; 2] = [".local/share/fonts", ".fonts"];

pub fn load_system_font() -> Option<Font> {
    let family = std::env::var(ENV_FONT_FAMILY).ok()?;
    let mut candidates = Vec::new();
    for directory in font_directories() {
        collect_font_files(&directory, &mut candidates);
    }
    candidates
        .into_iter()
        .find(|path| matches_family(path, &family))
        .and_then(|path| Font::from_path(&path).ok())
}

pub fn load_system_font_or_embedded() -> Font {
    load_system_font().unwrap_or_else(Font::embedded)
}

fn font_directories() -> Vec<PathBuf> {
    let mut directories = Vec::new();
    for directory in SYSTEM_FONT_DIRS {
        directories.push(PathBuf::from(directory));
    }
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        for subdirectory in USER_FONT_SUBDIRS {
            directories.push(home.join(subdirectory));
        }
    }
    directories
}

fn collect_font_files(directory: &Path, output: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_font_files(&path, output);
        } else if has_font_extension(&path) {
            output.push(path);
        }
    }
}

fn has_font_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| FONT_EXTENSIONS.iter().any(|known| known.eq_ignore_ascii_case(extension)))
        .unwrap_or(false)
}

fn matches_family(path: &Path, family: &str) -> bool {
    Font::peek_family(path)
        .map(|name| name.eq_ignore_ascii_case(family))
        .unwrap_or(false)
}
