use std::fs;
use std::path::{Path, PathBuf};

use crate::models::FileNode;

/// Recursively reads a directory and returns a tree of `FileNode`s.
/// Limits depth to prevent scanning enormous trees.
pub fn read_dir_tree(root: &Path, max_depth: usize) -> Vec<FileNode> {
    if max_depth == 0 {
        return Vec::new();
    }
    let mut nodes = Vec::new();

    let entries = match fs::read_dir(root) {
        Ok(e) => e,
        Err(_) => return nodes,
    };

    let mut entries_vec: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    entries_vec.sort_by_key(|e| {
        let is_file = e.file_type().map(|t| t.is_file()).unwrap_or(true);
        (is_file, e.file_name().to_ascii_lowercase())
    });

    for entry in entries_vec {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files/dirs and common noisy dirs
        if name.starts_with('.') || name == "node_modules" || name == "target" || name == "__pycache__" {
            continue;
        }

        let is_dir = path.is_dir();
        let children = if is_dir {
            read_dir_tree(&path, max_depth - 1)
        } else {
            Vec::new()
        };

        nodes.push(FileNode {
            name,
            path,
            is_dir,
            children,
            expanded: false,
        });
    }

    nodes
}

/// Read file content as a string. Returns error if not valid UTF-8 or too large.
pub fn read_file(path: &Path) -> Result<String, String> {
    let metadata = fs::metadata(path).map_err(|e| format!("Cannot read file: {}", e))?;
    if metadata.len() > 10 * 1024 * 1024 {
        return Err("File too large (>10MB)".to_string());
    }
    fs::read_to_string(path).map_err(|e| format!("Cannot read file: {}", e))
}

/// Write content to a file on disk.
pub fn write_file(path: &Path, content: &str) -> Result<(), String> {
    fs::write(path, content).map_err(|e| format!("Cannot write file: {}", e))
}

/// Create a backup copy of a file in `.devos_backups/` relative to project root.
pub fn create_backup(file_path: &Path, project_root: &Path) -> Result<PathBuf, String> {
    let relative = file_path
        .strip_prefix(project_root)
        .map_err(|_| "File is not within project root".to_string())?;

    let backup_dir = project_root.join(".devos_backups");
    let backup_path = backup_dir.join(relative);

    if let Some(parent) = backup_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create backup directory: {}", e))?;
    }

    fs::copy(file_path, &backup_path)
        .map_err(|e| format!("Cannot create backup: {}", e))?;

    Ok(backup_path)
}
