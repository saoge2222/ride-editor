use std::fs;
use std::path::{Path, PathBuf};

use super::render_font::Font;
use super::render_font_ttc::FontCollection;

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

pub fn load_cjk_font() -> Option<Font> {
    if let Ok(path) = std::env::var("RIDE_CJK_FONT") {
        let path = Path::new(&path);
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext.eq_ignore_ascii_case("ttc") {
                if let Ok(collection) = FontCollection::from_path(path) {
                    return collection.into_first_face();
                }
            }
        }
        if let Ok(font) = Font::from_path(path) {
            return Some(font);
        }
    }
    for directory in font_directories() {
        if let Some(font) = scan_cjk_dir(&directory) {
            return Some(font);
        }
    }
    None
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

fn is_cjk_filename(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("notosanscjk")
        || lower.contains("sourcehansans")
        || lower.contains("wenquanyi")
        || lower.starts_with("wqy")
        || lower.contains("cjk")
}

fn scan_cjk_dir(directory: &Path) -> Option<Font> {
    let entries = fs::read_dir(directory).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(font) = scan_cjk_dir(&path) {
                return Some(font);
            }
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !is_cjk_filename(name) {
            continue;
        }
        if name.to_lowercase().ends_with(".ttc") {
            if let Ok(collection) = FontCollection::from_path(&path) {
                return collection.into_first_face();
            }
        } else if has_font_extension(&path) {
            if let Ok(font) = Font::from_path(&path) {
                return Some(font);
            }
        }
    }
    None
}
