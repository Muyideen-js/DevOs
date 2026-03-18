use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct FileIndex {
    pub functions: Vec<String>,
    pub structs: Vec<String>,
    pub classes: Vec<String>,
}

pub type ProjectIndex = HashMap<PathBuf, FileIndex>;

/// Recursively walk a directory and build an index of code symbols.
pub fn build_project_index(root: &Path) -> ProjectIndex {
    let mut index = HashMap::new();
    walk_and_index(root, root, &mut index);
    index
}

fn walk_and_index(dir: &Path, root: &Path, index: &mut ProjectIndex) {
    if !dir.is_dir() {
        return;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Ignore common build/hidden dirs
            if name.starts_with('.') || name == "node_modules" || name == "target" || name == "dist" || name == "build" {
                continue;
            }

            if path.is_dir() {
                walk_and_index(&path, root, index);
            } else if path.is_file() {
                // Only index common source files
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ["rs", "js", "ts", "py", "go", "cpp", "c", "java"].contains(&ext) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let file_index = parse_file_symbols(&content, ext);
                        if !file_index.functions.is_empty() || !file_index.structs.is_empty() || !file_index.classes.is_empty() {
                            index.insert(path, file_index);
                        }
                    }
                }
            }
        }
    }
}

/// Very basic heuristic text parsing for common symbols.
fn parse_file_symbols(content: &str, ext: &str) -> FileIndex {
    let mut files_idx = FileIndex::default();

    for line in content.lines() {
        let trimmed = line.trim();
        
        // Skip comments
        if trimmed.starts_with("//") || trimmed.starts_with('#') {
            continue;
        }

        match ext {
            "rs" => {
                if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") {
                    if let Some(name) = extract_name_before_paren(trimmed, "fn ") {
                        files_idx.functions.push(name);
                    }
                } else if trimmed.starts_with("struct ") || trimmed.starts_with("pub struct ") {
                    if let Some(name) = extract_name_before_brace(trimmed, "struct ") {
                        files_idx.structs.push(name);
                    }
                } else if trimmed.starts_with("enum ") || trimmed.starts_with("pub enum ") {
                    if let Some(name) = extract_name_before_brace(trimmed, "enum ") {
                        files_idx.structs.push(name);
                    }
                }
            }
            "js" | "ts" => {
                if trimmed.starts_with("function ") || trimmed.starts_with("export function ") {
                    if let Some(name) = extract_name_before_paren(trimmed, "function ") {
                        files_idx.functions.push(name);
                    }
                } else if trimmed.starts_with("class ") || trimmed.starts_with("export class ") {
                    if let Some(name) = extract_name_before_brace(trimmed, "class ") {
                        files_idx.classes.push(name);
                    }
                }
            }
            "py" => {
                if trimmed.starts_with("def ") {
                    if let Some(name) = extract_name_before_paren(trimmed, "def ") {
                        files_idx.functions.push(name);
                    }
                } else if trimmed.starts_with("class ") {
                    if let Some(name) = extract_name_before_paren(trimmed, "class ") {
                        files_idx.classes.push(name);
                    } else if let Some(name) = extract_name_before_colon(trimmed, "class ") {
                        files_idx.classes.push(name);
                    }
                }
            }
            _ => {}
        }
    }

    files_idx
}

// Helpers
fn extract_name_before_paren(line: &str, keyword: &str) -> Option<String> {
    let after_kw = line.split(keyword).nth(1)?;
    let name = after_kw.split('(').next()?.split('<').next()?.trim();
    if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Some(name.to_string())
    } else {
        None
    }
}

fn extract_name_before_brace(line: &str, keyword: &str) -> Option<String> {
    let after_kw = line.split(keyword).nth(1)?;
    let name = after_kw.split('{').next()?.split('<').next()?.trim();
    if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Some(name.to_string())
    } else {
        None
    }
}

fn extract_name_before_colon(line: &str, keyword: &str) -> Option<String> {
    let after_kw = line.split(keyword).nth(1)?;
    let name = after_kw.split(':').next()?.trim();
    if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Some(name.to_string())
    } else {
        None
    }
}
