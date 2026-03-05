use egui::{self, Color32, RichText, TextEdit, Vec2};

use crate::core::file_manager;
use crate::state::AppState;

/// Render the tabbed editor panel in the center.
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    if state.editor.tabs.is_empty() {
        // Empty state
        ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new("Open a file from the explorer\nor press Ctrl+O to open a project")
                    .size(16.0)
                    .color(Color32::from_rgb(100, 100, 100)),
            );
        });
        return;
    }

    // Tab bar
    let mut tab_to_close: Option<usize> = None;
    let mut tab_to_activate: Option<usize> = None;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(2.0, 0.0);
        for (i, tab) in state.editor.tabs.iter().enumerate() {
            let is_active = state.editor.active_tab == Some(i);
            let title = tab.title();

            let bg = if is_active {
                Color32::from_rgb(45, 45, 48)
            } else {
                Color32::from_rgb(30, 30, 34)
            };

            let text_color = if tab.is_modified() {
                Color32::from_rgb(255, 200, 100)
            } else if is_active {
                Color32::WHITE
            } else {
                Color32::from_rgb(160, 160, 160)
            };

            let frame = egui::Frame::new()
                .fill(bg)
                .inner_margin(egui::Margin::symmetric(10, 4))
                .corner_radius(egui::CornerRadius {
                    nw: 4,
                    ne: 4,
                    sw: 0,
                    se: 0,
                });

            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(false, RichText::new(&title).color(text_color).size(12.0))
                        .clicked()
                    {
                        tab_to_activate = Some(i);
                    }

                    // Close button
                    if ui
                        .small_button(RichText::new("✕").size(10.0).color(Color32::from_rgb(150, 150, 150)))
                        .clicked()
                    {
                        tab_to_close = Some(i);
                    }
                });
            });
        }
    });

    // Handle tab actions
    if let Some(idx) = tab_to_activate {
        state.editor.active_tab = Some(idx);
    }
    if let Some(idx) = tab_to_close {
        state.editor.close_tab(idx);
    }

    ui.separator();

    // Editor content
    if let Some(active) = state.editor.active_tab {
        if active < state.editor.tabs.len() {
            // Search bar (Ctrl+F)
            if state.editor.show_search {
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    ui.text_edit_singleline(&mut state.editor.search_query);
                    if ui.button("✕").clicked() {
                        state.editor.show_search = false;
                        state.editor.search_query.clear();
                    }
                });
                ui.separator();
            }

            // File content editor
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let tab = &mut state.editor.tabs[active];

                    // Use monospace font for code
                    let text_edit = TextEdit::multiline(&mut tab.content)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .desired_rows(40)
                        .lock_focus(true);

                    ui.add(text_edit);
                });
        }
    }
}

/// Handle Ctrl+S to save the active file.
pub fn handle_save(state: &mut AppState) -> Option<String> {
    if let Some(active) = state.editor.active_tab {
        if active < state.editor.tabs.len() {
            let tab = &mut state.editor.tabs[active];
            match file_manager::write_file(&tab.path, &tab.content) {
                Ok(()) => {
                    tab.original = tab.content.clone();
                    return Some(format!("Saved {}", tab.label));
                }
                Err(e) => {
                    return Some(format!("Save failed: {}", e));
                }
            }
        }
    }
    None
}
