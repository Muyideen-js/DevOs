use egui::{self, Color32, RichText, Vec2};

use crate::state::AppState;
use crate::core::{file_manager, project};

/// Render the top bar with project name, open button, and settings.
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);

        // App logo / name
        ui.label(
            RichText::new("⚡ DevOS")
                .strong()
                .size(18.0)
                .color(Color32::from_rgb(100, 180, 255)),
        );

        ui.separator();

        // Current project path
        if let Some(ref root) = state.project.root {
            let display = root.display().to_string();
            let short = if display.len() > 50 {
                format!("...{}", &display[display.len() - 47..])
            } else {
                display
            };
            ui.label(
                RichText::new(format!("📁 {}", short))
                    .size(13.0)
                    .color(Color32::from_rgb(180, 180, 180)),
            );
        } else {
            ui.label(
                RichText::new("No project open")
                    .italics()
                    .size(13.0)
                    .color(Color32::from_rgb(120, 120, 120)),
            );
        }

        // Spacer
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Settings button
            if ui
                .button(RichText::new("⚙ Settings").size(13.0))
                .clicked()
            {
                state.settings.show_settings = !state.settings.show_settings;
            }

            // Terminal toggle
            let term_label = if state.terminal.show {
                "▼ Terminal"
            } else {
                "▲ Terminal"
            };
            if ui.button(RichText::new(term_label).size(13.0)).clicked() {
                state.terminal.show = !state.terminal.show;
            }

            // Open Project button
            if ui
                .button(RichText::new("📂 Open Project").size(13.0))
                .clicked()
            {
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
                    .selected_text(RichText::new("Recent").size(13.0))
                    .width(160.0)
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
