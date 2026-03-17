use egui::{self, Color32, RichText, Vec2};

use crate::state::AppState;
use crate::core::{file_manager, project};

/// Render the top bar with project name, open button, and settings.
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(10.0, 0.0);

        // App logo / name
        ui.label(
            RichText::new("⚡")
                .size(18.0)
                .color(Color32::from_rgb(80, 150, 240)),
        );
        ui.label(
            RichText::new("DevOS")
                .strong()
                .size(16.0)
                .color(Color32::from_rgb(200, 210, 230)),
        );

        // Thin separator
        ui.add_space(2.0);
        ui.label(
            RichText::new("│")
                .size(16.0)
                .color(Color32::from_rgb(45, 48, 58)),
        );
        ui.add_space(2.0);

        // Current project path
        if let Some(ref root) = state.project.root {
            let display = root.display().to_string();
            let short = if display.len() > 45 {
                format!("...{}", &display[display.len() - 42..])
            } else {
                display
            };
            ui.label(
                RichText::new(&short)
                    .size(12.0)
                    .color(Color32::from_rgb(130, 140, 155)),
            );
        }

        // Right side buttons
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);

            // Settings button
            let settings_btn = styled_topbar_btn(ui, "⚙", "Settings");
            if settings_btn.clicked() {
                state.settings.show_settings = !state.settings.show_settings;
            }

            // Terminal toggle
            let term_icon = if state.terminal.show { "▼" } else { "▶" };
            let term_btn = styled_topbar_btn(ui, term_icon, "Terminal");
            if term_btn.clicked() {
                state.terminal.show = !state.terminal.show;
            }

            // Open Project button
            let open_btn = styled_topbar_btn_accent(ui, "📂", "Open");
            if open_btn.clicked() {
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    let mut tree = file_manager::read_dir_tree(&folder, 8);
                    crate::ui::file_explorer::expand_first_level(&mut tree);
                    state.explorer.tree = tree;

                    // Update recent projects
                    project::add_recent_project(
                        &mut state.project.recent,
                        folder.clone(),
                    );
                    let _ = project::save_recent_projects(&state.project.recent);

                    state.project.root = Some(folder);

                    // Reset git state
                    state.git.is_repo = false;
                    state.git.branch = String::new();
                    state.git.files.clear();
                    state.git.commit_message.clear();

                    // Check git
                    if let Some(ref root) = state.project.root {
                        if let Some(repo) = crate::core::git::open_repo(root) {
                            state.git.is_repo = true;
                            state.git.branch = crate::core::git::get_branch(&repo);
                            state.git.files = crate::core::git::get_status(&repo);
                        }
                    }

                    // Update chat context file list
                    state.chat.context_files.clear();
                }
            }

            // Recent projects dropdown
            if !state.project.recent.is_empty() {
                egui::ComboBox::from_id_salt("recent_projects")
                    .selected_text(RichText::new("Recent").size(11.5).color(Color32::from_rgb(160, 165, 175)))
                    .width(130.0)
                    .show_ui(ui, |ui| {
                        let recent_clone = state.project.recent.clone();
                        for p in &recent_clone {
                            let label = p
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| p.display().to_string());
                            if ui.selectable_label(false, &label).clicked() {
                                let mut tree = file_manager::read_dir_tree(p, 8);
                                crate::ui::file_explorer::expand_first_level(&mut tree);
                                state.explorer.tree = tree;
                                state.project.root = Some(p.clone());
                            }
                        }
                    });
            }
        });
    });
}

// ── Styled Buttons ─────────────────────────────────────────────

fn styled_topbar_btn(ui: &mut egui::Ui, icon: &str, label: &str) -> egui::Response {
    let btn_text = format!("{} {}", icon, label);
    let btn = egui::Button::new(
        RichText::new(btn_text)
            .size(11.5)
            .color(Color32::from_rgb(170, 178, 192)),
    )
    .fill(Color32::from_rgb(32, 34, 42))
    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(50, 52, 62)))
    .corner_radius(4);

    ui.add(btn)
}

fn styled_topbar_btn_accent(ui: &mut egui::Ui, icon: &str, label: &str) -> egui::Response {
    let btn_text = format!("{} {}", icon, label);
    let btn = egui::Button::new(
        RichText::new(btn_text)
            .size(11.5)
            .color(Color32::from_rgb(200, 215, 240)),
    )
    .fill(Color32::from_rgb(35, 55, 90))
    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(60, 90, 140)))
    .corner_radius(4);

    ui.add(btn)
}
