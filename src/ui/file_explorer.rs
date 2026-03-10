use egui::{self, Color32, RichText};
use std::path::PathBuf;

use crate::core::file_manager;
use crate::models::FileNode;
use crate::state::AppState;

/// Render the file explorer tree in the left side panel.
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("  EXPLORER")
                .size(11.0)
                .strong()
                .color(Color32::from_rgb(120, 140, 160)),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !state.explorer.tree.is_empty() {
                if ui
                    .small_button(RichText::new("⟳").size(12.0))
                    .on_hover_text("Refresh file tree")
                    .clicked()
                {
                    if let Some(ref root) = state.project.root {
                        state.explorer.tree = file_manager::read_dir_tree(root, 8);
                        expand_first_level(&mut state.explorer.tree);
                    }
                }
            }
        });
    });

    ui.add_space(2.0);

    if state.explorer.tree.is_empty() {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("📂")
                    .size(32.0)
                    .color(Color32::from_rgb(80, 80, 90)),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new("No project open")
                    .size(13.0)
                    .color(Color32::from_rgb(90, 90, 100)),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new("Click \"Open Project\" above")
                    .italics()
                    .size(11.0)
                    .color(Color32::from_rgb(70, 70, 80)),
            );
        });
        return;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // We collect actions to apply after rendering to satisfy borrow checker
            let mut dir_to_toggle: Option<PathBuf> = None;
            let mut file_to_open: Option<PathBuf> = None;

            render_tree(ui, &state.explorer.tree, 0, &mut dir_to_toggle, &mut file_to_open);

            // Apply directory toggle
            if let Some(path) = dir_to_toggle {
                toggle_dir(&mut state.explorer.tree, &path);
            }

            // Apply file open
            if let Some(path) = file_to_open {
                match file_manager::read_file(&path) {
                    Ok(content) => {
                        state.editor.open_file(path, content);
                    }
                    Err(e) => {
                        eprintln!("Failed to read file: {}", e);
                    }
                }
            }
        });
}

/// Render the tree recursively, collecting click actions.
fn render_tree(
    ui: &mut egui::Ui,
    nodes: &[FileNode],
    depth: usize,
    dir_to_toggle: &mut Option<PathBuf>,
    file_to_open: &mut Option<PathBuf>,
) {
    let indent = depth as f32 * 14.0;

    for node in nodes {
        ui.horizontal(|ui| {
            ui.add_space(indent);

            if node.is_dir {
                let arrow = if node.expanded { "▼" } else { "▶" };
                let icon = if node.expanded { "📂" } else { "📁" };
                let label = format!("{} {} {}", arrow, icon, node.name);

                let color = if node.expanded {
                    Color32::from_rgb(200, 210, 220)
                } else {
                    Color32::from_rgb(170, 180, 190)
                };

                let response = ui.selectable_label(
                    false,
                    RichText::new(label).size(13.0).color(color),
                );

                if response.clicked() {
                    *dir_to_toggle = Some(node.path.clone());
                }
            } else {
                let icon = file_icon(&node.name);
                let label = format!("   {} {}", icon, node.name);
                let color = file_color(&node.name);

                let response = ui.selectable_label(
                    false,
                    RichText::new(label).size(12.5).color(color),
                );

                if response.clicked() {
                    *file_to_open = Some(node.path.clone());
                }
            }
        });

        // Render children if expanded
        if node.is_dir && node.expanded {
            render_tree(ui, &node.children, depth + 1, dir_to_toggle, file_to_open);
        }
    }
}

/// Toggle expanded state of a directory node by path.
fn toggle_dir(tree: &mut Vec<FileNode>, target: &std::path::Path) -> bool {
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

/// Auto-expand the first level of directories after opening a project.
pub fn expand_first_level(tree: &mut Vec<FileNode>) {
    for node in tree.iter_mut() {
        if node.is_dir {
            node.expanded = true;
        }
    }
}

/// Get an icon for a file based on extension.
fn file_icon(name: &str) -> &'static str {
    if let Some(ext) = name.rsplit('.').next() {
        match ext.to_lowercase().as_str() {
            "rs" => "🦀",
            "toml" | "yaml" | "yml" => "⚙",
            "json" => "{}",
            "md" | "txt" | "doc" => "📝",
            "py" => "🐍",
            "js" | "jsx" => "📜",
            "ts" | "tsx" => "📘",
            "html" | "htm" => "🌐",
            "css" | "scss" | "sass" => "🎨",
            "png" | "jpg" | "jpeg" | "svg" | "gif" | "ico" => "🖼",
            "lock" => "🔒",
            "sh" | "bash" | "zsh" | "bat" | "ps1" => "⚡",
            "env" => "🔑",
            "sql" => "🗃",
            "xml" => "📋",
            _ => "📄",
        }
    } else if name.starts_with('.') {
        "⚙"
    } else {
        "📄"
    }
}

/// Get color for a file based on extension.
fn file_color(name: &str) -> Color32 {
    if let Some(ext) = name.rsplit('.').next() {
        match ext.to_lowercase().as_str() {
            "rs" => Color32::from_rgb(220, 140, 60),
            "toml" | "yaml" | "yml" | "json" => Color32::from_rgb(180, 180, 120),
            "md" | "txt" => Color32::from_rgb(140, 180, 220),
            "py" => Color32::from_rgb(80, 180, 80),
            "js" | "jsx" => Color32::from_rgb(240, 210, 80),
            "ts" | "tsx" => Color32::from_rgb(70, 140, 220),
            "html" | "htm" => Color32::from_rgb(230, 120, 60),
            "css" | "scss" => Color32::from_rgb(100, 160, 230),
            "lock" => Color32::from_rgb(120, 120, 120),
            _ => Color32::from_rgb(190, 195, 200),
        }
    } else {
        Color32::from_rgb(190, 195, 200)
    }
}
