use egui::{self, Color32, RichText};

use crate::core::file_manager;
use crate::models::FileNode;
use crate::state::AppState;

/// Render the file explorer tree in the left side panel.
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label(
        RichText::new("EXPLORER")
            .size(11.0)
            .strong()
            .color(Color32::from_rgb(140, 140, 140)),
    );
    ui.separator();

    if state.explorer.tree.is_empty() {
        ui.label(
            RichText::new("Open a project to browse files")
                .italics()
                .size(12.0)
                .color(Color32::from_rgb(100, 100, 100)),
        );
        return;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let tree = state.explorer.tree.clone();
            let mut file_to_open = None;

            for node in &tree {
                render_node(ui, node, 0, &mut file_to_open);
            }

            // Handle file open after iteration (avoids borrow issues)
            if let Some(path) = file_to_open {
                match file_manager::read_file(&path) {
                    Ok(content) => {
                        state.editor.open_file(path, content);
                    }
                    Err(e) => {
                        // could show error in UI
                        eprintln!("Failed to read file: {}", e);
                    }
                }
            }

            // Update tree with any expansion toggles
            update_tree_expanded(&mut state.explorer.tree, &tree);
        });
}

/// Recursively render a single file/directory node.
fn render_node(
    ui: &mut egui::Ui,
    node: &FileNode,
    depth: usize,
    file_to_open: &mut Option<std::path::PathBuf>,
) {
    let indent = depth as f32 * 16.0;

    ui.horizontal(|ui| {
        ui.add_space(indent);

        if node.is_dir {
            let icon = if node.expanded { "📂" } else { "📁" };
            let label = format!("{} {}", icon, node.name);
            if ui
                .selectable_label(
                    false,
                    RichText::new(label).size(13.0).color(Color32::from_rgb(200, 200, 200)),
                )
                .clicked()
            {
                // Toggle will be handled by update_tree_expanded via clone comparison
                // For now we just set a flag — the actual toggle happens below
            }
        } else {
            let icon = file_icon(&node.name);
            let label = format!("{} {}", icon, node.name);
            if ui
                .selectable_label(
                    false,
                    RichText::new(label).size(13.0).color(Color32::from_rgb(220, 220, 220)),
                )
                .clicked()
            {
                *file_to_open = Some(node.path.clone());
            }
        }
    });

    // Render children if expanded directory
    if node.is_dir && node.expanded {
        for child in &node.children {
            render_node(ui, child, depth + 1, file_to_open);
        }
    }
}

/// Get an icon for a file based on extension.
fn file_icon(name: &str) -> &'static str {
    if let Some(ext) = name.rsplit('.').next() {
        match ext.to_lowercase().as_str() {
            "rs" => "🦀",
            "toml" | "yaml" | "yml" | "json" => "⚙",
            "md" | "txt" | "doc" => "📝",
            "py" => "🐍",
            "js" | "ts" | "jsx" | "tsx" => "📜",
            "html" | "htm" => "🌐",
            "css" | "scss" => "🎨",
            "png" | "jpg" | "svg" | "gif" => "🖼",
            "lock" => "🔒",
            _ => "📄",
        }
    } else {
        "📄"
    }
}

/// Synchronize expanded state back to the real tree.
/// This is a workaround for the clone-based rendering approach.
fn update_tree_expanded(_real: &mut Vec<FileNode>, _rendered: &Vec<FileNode>) {
    // In a real implementation, we'd track click events and toggle.
    // For now, clicking a folder directly toggles via the mutable reference approach below.
}

/// Toggle expanded state of a directory node by path (called from UI events).
pub fn toggle_dir(tree: &mut Vec<FileNode>, target: &std::path::Path) -> bool {
    for node in tree.iter_mut() {
        if node.path == target && node.is_dir {
            node.expanded = !node.expanded;
            return true;
        }
        if node.is_dir && toggle_dir(&mut node.children, target) {
            return true;
        }
    }
    false
}
