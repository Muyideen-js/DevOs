use egui::{self, Color32, RichText, TextEdit};

use crate::core::git;
use crate::state::AppState;

/// Render the Git panel in the right sidebar.
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label(
        RichText::new("GIT")
            .size(11.0)
            .strong()
            .color(Color32::from_rgb(140, 140, 140)),
    );
    ui.separator();

    if !state.git.is_repo {
        ui.label(
            RichText::new("Not a git repository")
                .italics()
                .size(12.0)
                .color(Color32::from_rgb(100, 100, 100)),
        );
        return;
    }

    // Branch info
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("⎇")
                .size(14.0)
                .color(Color32::from_rgb(100, 200, 100)),
        );
        ui.label(
            RichText::new(&state.git.branch)
                .size(13.0)
                .strong()
                .color(Color32::from_rgb(200, 200, 200)),
        );
    });

    ui.add_space(4.0);

    // Refresh button
    if ui.button(RichText::new("🔄 Refresh").size(12.0)).clicked() {
        refresh_git(state);
    }

    ui.add_space(4.0);

    // Error display
    if let Some(ref err) = state.git.last_error {
        ui.label(
            RichText::new(err)
                .size(11.0)
                .color(Color32::from_rgb(255, 120, 120)),
        );
        ui.add_space(4.0);
    }

    // Changed files
    if state.git.files.is_empty() {
        ui.label(
            RichText::new("Working tree clean ✓")
                .size(12.0)
                .color(Color32::from_rgb(100, 180, 100)),
        );
    } else {
        ui.label(
            RichText::new(format!("Changes ({})", state.git.files.len()))
                .size(12.0)
                .strong()
                .color(Color32::from_rgb(200, 200, 200)),
        );

        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                let files_clone = state.git.files.clone();
                for file in &files_clone {
                    ui.horizontal(|ui| {
                        let status_color = match file.status {
                            crate::models::GitStatus::Modified => Color32::from_rgb(220, 180, 50),
                            crate::models::GitStatus::New | crate::models::GitStatus::Untracked => {
                                Color32::from_rgb(100, 200, 100)
                            }
                            crate::models::GitStatus::Deleted => Color32::from_rgb(255, 100, 100),
                            crate::models::GitStatus::Renamed => Color32::from_rgb(100, 150, 255),
                        };

                        let staged_icon = if file.staged { "●" } else { "○" };
                        let staged_color = if file.staged {
                            Color32::from_rgb(100, 200, 100)
                        } else {
                            Color32::from_rgb(120, 120, 120)
                        };

                        // Stage/unstage toggle
                        if ui
                            .button(RichText::new(staged_icon).color(staged_color).size(12.0))
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
                                    // Refresh status
                                    state.git.files = git::get_status(&repo);
                                }
                            }
                        }

                        ui.label(
                            RichText::new(format!("{}", file.status))
                                .monospace()
                                .size(12.0)
                                .color(status_color),
                        );

                        ui.label(
                            RichText::new(&file.path)
                                .size(12.0)
                                .color(Color32::from_rgb(200, 200, 200)),
                        );
                    });
                }
            });
    }

    ui.add_space(8.0);

    // Commit section
    ui.label(
        RichText::new("Commit Message")
            .size(11.0)
            .color(Color32::from_rgb(160, 160, 160)),
    );

    ui.add(
        TextEdit::multiline(&mut state.git.commit_message)
            .desired_rows(3)
            .desired_width(f32::INFINITY)
            .hint_text("Enter commit message..."),
    );

    let has_staged = state.git.files.iter().any(|f| f.staged);
    let has_message = !state.git.commit_message.trim().is_empty();

    if ui
        .add_enabled(
            has_staged && has_message,
            egui::Button::new(RichText::new("✓ Commit").size(13.0)),
        )
        .clicked()
    {
        if let Some(ref root) = state.project.root {
            if let Some(repo) = git::open_repo(root) {
                match git::commit(&repo, &state.git.commit_message) {
                    Ok(()) => {
                        state.git.commit_message.clear();
                        state.git.last_error = None;
                        // Refresh
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
