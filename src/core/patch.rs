use std::path::Path;

use crate::core::file_manager;
use crate::models::FilePatch;

/// Parse a unified diff text and extract file patches.
/// Expects standard unified diff format with --- and +++ headers.
pub fn parse_unified_diff(diff_text: &str) -> Vec<FilePatch> {
    let mut patches = Vec::new();
    let mut current_file: Option<String> = None;
    let mut current_diff = String::new();

    for line in diff_text.lines() {
        if line.starts_with("--- a/") || line.starts_with("--- ") {
            // If we have a previous patch, save it
            if let Some(ref file) = current_file {
                if !current_diff.is_empty() {
                    patches.push(FilePatch {
                        file_path: file.clone(),
                        original: String::new(), // filled in on apply
                        patched: String::new(),   // filled in on apply
                        diff_text: current_diff.clone(),
                    });
                }
            }
            current_diff = String::new();
            current_diff.push_str(line);
            current_diff.push('\n');
        } else if line.starts_with("+++ b/") || line.starts_with("+++ ") {
            let file_path = line
                .trim_start_matches("+++ b/")
                .trim_start_matches("+++ ")
                .to_string();
            current_file = Some(file_path);
            current_diff.push_str(line);
            current_diff.push('\n');
        } else if current_file.is_some() {
            current_diff.push_str(line);
            current_diff.push('\n');
        }
    }

    // Save last patch
    if let Some(ref file) = current_file {
        if !current_diff.is_empty() {
            patches.push(FilePatch {
                file_path: file.clone(),
                original: String::new(),
                patched: String::new(),
                diff_text: current_diff,
            });
        }
    }

    patches
}

/// Apply a patch to a file. Creates a backup first.
/// Uses the `diffy` crate for proper diff application.
pub fn apply_patch(patch: &FilePatch, project_root: &Path) -> Result<String, String> {
    let file_path = project_root.join(&patch.file_path);

    // Read original content
    let original = if file_path.exists() {
        file_manager::read_file(&file_path)?
    } else {
        String::new()
    };

    // Create backup before modifying
    if file_path.exists() {
        file_manager::create_backup(&file_path, project_root)?;
    }

    // Apply using diffy
    let applied = diffy::apply(&original, &diffy::Patch::from_str(&patch.diff_text).map_err(|e| format!("Invalid patch: {}", e))?)
        .map_err(|e| format!("Patch apply failed: {}", e))?;

    // Write the patched content
    file_manager::write_file(&file_path, &applied)?;

    Ok(applied)
}

/// Simple text-based patch application as a fallback.
/// Replaces the original content lines with patched lines.
pub fn apply_patch_simple(
    file_path: &Path,
    _original_content: &str,
    new_content: &str,
    project_root: &Path,
) -> Result<(), String> {
    // Create backup
    if file_path.exists() {
        file_manager::create_backup(file_path, project_root)?;
    }

    // Write new content
    file_manager::write_file(file_path, new_content)
}
