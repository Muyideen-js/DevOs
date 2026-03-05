use egui::{self, Color32, RichText};

use crate::core::patch;
use crate::models::PatchAction;
use crate::state::AppState;

/// Render the patch preview panel (shown when AI proposes changes).
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    if state.patch.patches.is_empty() {
        ui.label(
            RichText::new("No patches to review")
                .italics()
                .size(12.0)
                .color(Color32::from_rgb(100, 100, 100)),
        );
        return;
    }

    ui.label(
        RichText::new("PATCH PREVIEW")
            .size(11.0)
            .strong()
            .color(Color32::from_rgb(140, 140, 140)),
    );
    ui.separator();

    let patches_len = state.patch.patches.len();

    // Ensure actions vec matches patches
    while state.patch.actions.len() < patches_len {
        state.patch.actions.push(PatchAction::Pending);
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for i in 0..patches_len {
                // Clone what we need to avoid borrow issues
                let file_path = state.patch.patches[i].file_path.clone();
                let diff_text = state.patch.patches[i].diff_text.clone();
                let action = state.patch.actions[i].clone();

                ui.group(|ui| {
                    // File header
                    ui.label(
                        RichText::new(format!("📄 {}", file_path))
                            .strong()
                            .size(13.0)
                            .color(Color32::from_rgb(200, 200, 200)),
                    );

                    // Diff content
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .id_salt(format!("diff_{}", i))
                        .show(ui, |ui| {
                            for line in diff_text.lines() {
                                let color = if line.starts_with('+') {
                                    Color32::from_rgb(100, 220, 100)
                                } else if line.starts_with('-') {
                                    Color32::from_rgb(255, 100, 100)
                                } else if line.starts_with("@@") {
                                    Color32::from_rgb(100, 150, 255)
                                } else {
                                    Color32::from_rgb(180, 180, 180)
                                };

                                ui.label(
                                    RichText::new(line)
                                        .monospace()
                                        .size(11.0)
                                        .color(color),
                                );
                            }
                        });

                    // Action buttons
                    match action {
                        PatchAction::Pending => {
                            ui.horizontal(|ui| {
                                if ui
                                    .button(
                                        RichText::new("✓ Apply")
                                            .size(12.0)
                                            .color(Color32::from_rgb(100, 220, 100)),
                                    )
                                    .clicked()
                                {
                                    if let Some(ref root) = state.project.root {
                                        let p = state.patch.patches[i].clone();
                                        match patch::apply_patch(&p, root) {
                                            Ok(_) => {
                                                state.patch.actions[i] = PatchAction::Applied;
                                            }
                                            Err(e) => {
                                                eprintln!("Patch apply error: {}", e);
                                            }
                                        }
                                    }
                                }

                                if ui
                                    .button(
                                        RichText::new("✕ Reject")
                                            .size(12.0)
                                            .color(Color32::from_rgb(255, 100, 100)),
                                    )
                                    .clicked()
                                {
                                    state.patch.actions[i] = PatchAction::Rejected;
                                }
                            });
                        }
                        PatchAction::Applied => {
                            ui.label(
                                RichText::new("✓ Applied")
                                    .size(12.0)
                                    .color(Color32::from_rgb(100, 220, 100)),
                            );
                        }
                        PatchAction::Rejected => {
                            ui.label(
                                RichText::new("✕ Rejected")
                                    .size(12.0)
                                    .color(Color32::from_rgb(255, 100, 100)),
                            );
                        }
                    }
                });

                ui.add_space(4.0);
            }

            // Clear all button
            if state.patch.actions.iter().all(|a| *a != PatchAction::Pending) {
                if ui.button("Clear Patches").clicked() {
                    state.patch.patches.clear();
                    state.patch.actions.clear();
                }
            }
        });
}
