use egui::{self, Color32, RichText, Vec2};
use std::path::PathBuf;

use crate::core::file_manager;
use crate::models::FileNode;
use crate::state::AppState;

// ── Color Palette ──────────────────────────────────────────────

struct Palette;
impl Palette {
    const HEADER:      Color32 = Color32::from_rgb(140, 155, 175);
    const DIR_ARROW:   Color32 = Color32::from_rgb(100, 110, 125);
    const DIR_NAME:    Color32 = Color32::from_rgb(195, 200, 210);
    const DIR_HOVER:   Color32 = Color32::from_rgb(210, 220, 235);
    const FILE_DIM:    Color32 = Color32::from_rgb(145, 150, 160);
    const EMPTY:       Color32 = Color32::from_rgb(80, 85, 95);
    const EMPTY_HINT:  Color32 = Color32::from_rgb(65, 70, 78);
    const REFRESH_BTN: Color32 = Color32::from_rgb(100, 155, 220);
}

/// Render the file explorer tree in the left side panel.
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    // ── Header bar ──
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("EXPLORER")
                .size(10.5)
                .strong()
                .color(Palette::HEADER),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !state.explorer.tree.is_empty() {
                if ui
                    .small_button(RichText::new("⟳").size(13.0).color(Palette::REFRESH_BTN))
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

    ui.add_space(4.0);

    // ── Empty state ──
    if state.explorer.tree.is_empty() {
        ui.add_space(ui.available_height() * 0.25);
        ui.vertical_centered(|ui| {
            // Folder icon
            ui.label(
                RichText::new("📁")
                    .size(36.0)
                    .color(Palette::EMPTY),
            );
            ui.add_space(10.0);
            ui.label(
                RichText::new("No project open")
                    .size(13.0)
                    .strong()
                    .color(Palette::EMPTY),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new("Open a folder to get started")
                    .italics()
                    .size(11.0)
                    .color(Palette::EMPTY_HINT),
            );
        });
        return;
    }

    // ── Project name header ──
    if let Some(ref root) = state.project.root {
        let proj_name = root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Project".into());

        egui::Frame::new()
            .fill(Color32::from_rgb(28, 30, 38))
            .inner_margin(egui::Margin::symmetric(6, 3))
            .corner_radius(3)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(format!("  {}", proj_name.to_uppercase()))
                        .size(10.0)
                        .strong()
                        .color(Color32::from_rgb(120, 135, 160)),
                );
            });
        ui.add_space(4.0);
    }

    // ── File tree ──
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
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
    let indent = depth as f32 * 16.0;

    for node in nodes {
        if node.is_dir {
            // ── Directory row ──
            let arrow = if node.expanded { "▾" } else { "▸" };
            let icon = if node.expanded { "▼" } else { "▶" };

            let response = ui.horizontal(|ui| {
                ui.add_space(indent);
                ui.spacing_mut().item_spacing = Vec2::new(3.0, 0.0);

                // Arrow indicator
                ui.label(
                    RichText::new(arrow)
                        .size(10.0)
                        .color(Palette::DIR_ARROW),
                );

                // Folder icon - using text-based clean icons
                let folder_color = if node.expanded {
                    Color32::from_rgb(90, 160, 240)
                } else {
                    Color32::from_rgb(110, 140, 180)
                };
                ui.label(
                    RichText::new(if node.expanded { "⏷" } else { "⏵" })
                        .size(11.0)
                        .color(folder_color),
                );

                // Name
                let name_color = if node.expanded {
                    Palette::DIR_HOVER
                } else {
                    Palette::DIR_NAME
                };

                ui.selectable_label(
                    false,
                    RichText::new(&node.name).size(12.5).color(name_color),
                )
            });

            if response.inner.clicked() {
                *dir_to_toggle = Some(node.path.clone());
            }
        } else {
            // ── File row ──
            let ext = get_ext(&node.name);
            let (icon, icon_color) = file_type_icon(&ext);
            let name_color = file_type_name_color(&ext);

            let response = ui.horizontal(|ui| {
                ui.add_space(indent + 18.0); // extra indent for files under folders
                ui.spacing_mut().item_spacing = Vec2::new(5.0, 0.0);

                // Colored dot / symbol icon
                ui.label(
                    RichText::new(icon)
                        .size(11.0)
                        .color(icon_color),
                );

                // Name
                ui.selectable_label(
                    false,
                    RichText::new(&node.name).size(12.0).color(name_color),
                )
            });

            if response.inner.clicked() {
                *file_to_open = Some(node.path.clone());
            }
        }

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

// ── File type helpers ──────────────────────────────────────────

fn get_ext(name: &str) -> String {
    name.rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase()
}

/// Returns (icon_char, icon_color) for a file extension.
fn file_type_icon(ext: &str) -> (&'static str, Color32) {
    match ext {
        "rs"                    => ("◆", Color32::from_rgb(222, 143, 60)),   // Rust orange
        "py"                    => ("◆", Color32::from_rgb(55, 165, 85)),    // Python green
        "js" | "jsx" | "mjs"   => ("◆", Color32::from_rgb(245, 215, 75)),   // JS yellow
        "ts" | "tsx"            => ("◆", Color32::from_rgb(50, 130, 225)),   // TS blue
        "html" | "htm"          => ("◇", Color32::from_rgb(235, 105, 55)),   // HTML orange
        "css" | "scss" | "sass" => ("◇", Color32::from_rgb(85, 155, 235)),   // CSS blue
        "json"                  => ("◈", Color32::from_rgb(185, 185, 80)),   // JSON yellow-green
        "toml"                  => ("◈", Color32::from_rgb(160, 130, 200)),  // TOML purple
        "yaml" | "yml"          => ("◈", Color32::from_rgb(160, 130, 200)),
        "md" | "txt" | "doc"    => ("◇", Color32::from_rgb(115, 170, 225)),  // Docs blue
        "lock"                  => ("◆", Color32::from_rgb(90, 92, 100)),    // Lock dim
        "sh" | "bash" | "bat" | "ps1" => ("▸", Color32::from_rgb(100, 200, 130)),
        "png" | "jpg" | "jpeg" | "svg" | "gif" | "ico" => ("◆", Color32::from_rgb(200, 130, 200)),
        "env"                   => ("◆", Color32::from_rgb(200, 180, 80)),
        "xml"                   => ("◇", Color32::from_rgb(180, 140, 100)),
        "sql"                   => ("◈", Color32::from_rgb(140, 180, 220)),
        "gitignore"             => ("◆", Color32::from_rgb(250, 90, 80)),
        _                       => ("○", Color32::from_rgb(120, 125, 135)),
    }
}

/// Returns name color for file type.
fn file_type_name_color(ext: &str) -> Color32 {
    match ext {
        "rs"                    => Color32::from_rgb(215, 175, 120),
        "py"                    => Color32::from_rgb(150, 200, 160),
        "js" | "jsx" | "mjs"   => Color32::from_rgb(220, 210, 150),
        "ts" | "tsx"            => Color32::from_rgb(140, 180, 230),
        "html" | "htm"          => Color32::from_rgb(215, 155, 120),
        "css" | "scss" | "sass" => Color32::from_rgb(140, 185, 235),
        "json" | "toml" | "yaml" | "yml" => Color32::from_rgb(190, 185, 150),
        "md" | "txt"            => Color32::from_rgb(160, 185, 215),
        "lock"                  => Color32::from_rgb(110, 112, 118),
        _                       => Color32::from_rgb(175, 180, 190),
    }
}
