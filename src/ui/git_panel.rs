use egui::{self, Color32, RichText, TextEdit};

use crate::core::git;
use crate::state::AppState;

/// Render the Git panel in the right sidebar.
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label(
        RichText::new("SOURCE CONTROL")
            .size(10.5)
            .strong()
            .color(Color32::from_rgb(140, 155, 175)),
    );
    ui.add_space(4.0);

    if !state.git.is_repo {
        ui.add_space(20.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("Not a git repository")
                    .italics()
                    .size(12.0)
                    .color(Color32::from_rgb(90, 95, 105)),
            );
        });
        return;
    }

    // Branch info
    egui::Frame::new()
        .fill(Color32::from_rgb(28, 30, 38))
        .corner_radius(4)
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("⎇")
                        .size(13.0)
                        .color(Color32::from_rgb(80, 180, 120)),
                );
                ui.label(
                    RichText::new(&state.git.branch)
                        .size(12.5)
                        .strong()
                        .color(Color32::from_rgb(200, 210, 225)),
                );
            });
        });

    ui.add_space(6.0);

    // Refresh button
    let refresh_btn = egui::Button::new(
        RichText::new("⟳ Refresh")
            .size(11.0)
            .color(Color32::from_rgb(130, 170, 220)),
    )
    .fill(Color32::from_rgb(30, 35, 45))
    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(45, 55, 70)))
    .corner_radius(3);

    if ui.add(refresh_btn).clicked() {
        refresh_git(state);
    }

    ui.add_space(6.0);

    // Error display
    if let Some(ref err) = state.git.last_error {
        egui::Frame::new()
            .fill(Color32::from_rgb(60, 25, 25))
            .corner_radius(3)
            .inner_margin(egui::Margin::symmetric(6, 3))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(err)
                        .size(11.0)
                        .color(Color32::from_rgb(255, 140, 140)),
                );
            });
        ui.add_space(4.0);
    }

    // Changed files
    if state.git.files.is_empty() {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("✓")
                    .size(13.0)
                    .color(Color32::from_rgb(80, 200, 120)),
            );
            ui.label(
                RichText::new("Working tree clean")
                    .size(12.0)
                    .color(Color32::from_rgb(120, 180, 140)),
            );
        });
    } else {
        ui.label(
            RichText::new(format!("Changes ({})", state.git.files.len()))
                .size(11.0)
                .strong()
                .color(Color32::from_rgb(170, 175, 185)),
        );
        ui.add_space(2.0);

        egui::ScrollArea::vertical()
            .max_height(180.0)
            .show(ui, |ui| {
                let files_clone = state.git.files.clone();
                for file in &files_clone {
                    ui.horizontal(|ui| {
                        let status_color = match file.status {
                            crate::models::GitStatus::Modified => Color32::from_rgb(230, 190, 60),
                            crate::models::GitStatus::New | crate::models::GitStatus::Untracked => {
                                Color32::from_rgb(80, 200, 120)
                            }
                            crate::models::GitStatus::Deleted => Color32::from_rgb(240, 90, 90),
                            crate::models::GitStatus::Renamed => Color32::from_rgb(100, 150, 255),
                        };

                        // Stage/unstage toggle
                        let staged_indicator = if file.staged { "●" } else { "○" };
                        let staged_color = if file.staged {
                            Color32::from_rgb(80, 200, 120)
                        } else {
                            Color32::from_rgb(80, 82, 90)
                        };

                        let stage_btn = egui::Button::new(
                            RichText::new(staged_indicator).color(staged_color).size(11.0)
                        )
                        .fill(Color32::TRANSPARENT)
                        .stroke(egui::Stroke::NONE);

                        if ui.add(stage_btn)
                            .on_hover_text(if file.staged { "Unstage" } else { "Stage" })
                            .clicked()
                        {
                            if let Some(ref root) = state.project.root {
                                if let Some(repo) = git::open_repo(root) {
                                    let result = if file.staged {
                                        git::unstage_file(&repo, &file.path)
                                    } else {
                                        git::stage_file(&repo, &file.path)
                                    };
                                    match result {
                                        Ok(()) => {
                                            state.git.last_error = None;
                                        }
                                        Err(e) => {
                                            state.git.last_error = Some(e);
                                        }
                                    }
                                    state.git.files = git::get_status(&repo);
                                }
                            }
                        }

                        ui.label(
                            RichText::new(format!("{}", file.status))
                                .monospace()
                                .size(11.0)
                                .color(status_color),
                        );

                        ui.label(
                            RichText::new(&file.path)
                                .size(11.5)
                                .color(Color32::from_rgb(175, 180, 195)),
                        );
                    });
                }
            });
    }

    ui.add_space(10.0);

    // Commit section
    ui.label(
        RichText::new("Commit")
            .size(11.0)
            .strong()
            .color(Color32::from_rgb(140, 145, 155)),
    );
    ui.add_space(2.0);

    ui.add(
        TextEdit::multiline(&mut state.git.commit_message)
            .desired_rows(3)
            .desired_width(f32::INFINITY)
            .font(egui::TextStyle::Monospace)
            .hint_text("Commit message...")
            .text_color(Color32::from_rgb(200, 205, 215)),
    );
    ui.add_space(4.0);

    let has_staged = state.git.files.iter().any(|f| f.staged);
    let has_message = !state.git.commit_message.trim().is_empty();

    let commit_btn = egui::Button::new(
        RichText::new("✓ Commit")
            .size(12.0)
            .color(if has_staged && has_message {
                Color32::from_rgb(200, 220, 255)
            } else {
                Color32::from_rgb(80, 85, 95)
            }),
    )
    .fill(if has_staged && has_message {
        Color32::from_rgb(30, 60, 100)
    } else {
        Color32::from_rgb(30, 32, 38)
    })
    .stroke(egui::Stroke::new(
        1.0,
        if has_staged && has_message {
            Color32::from_rgb(50, 90, 150)
        } else {
            Color32::from_rgb(40, 42, 50)
        },
    ))
    .corner_radius(4);

    if ui.add_enabled(has_staged && has_message, commit_btn).clicked() {
        if let Some(ref root) = state.project.root {
            if let Some(repo) = git::open_repo(root) {
                match git::commit(&repo, &state.git.commit_message) {
                    Ok(()) => {
                        state.git.commit_message.clear();
                        state.git.last_error = None;
                        state.git.branch = git::get_branch(&repo);
                        state.git.files = git::get_status(&repo);
                    }
                    Err(e) => {
                        state.git.last_error = Some(e);
                    }
                }
            }
        }
    }
}

/// Refresh git status from the repository.
pub fn refresh_git(state: &mut AppState) {
    if let Some(ref root) = state.project.root {
        if let Some(repo) = git::open_repo(root) {
            state.git.is_repo = true;
            state.git.branch = git::get_branch(&repo);
            state.git.files = git::get_status(&repo);
            state.git.last_error = None;
        } else {
            state.git.is_repo = false;
        }
    }
}
