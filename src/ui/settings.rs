use egui::{self, Color32, RichText, TextEdit};

use crate::core::chat;
use crate::state::AppState;

/// Render the settings modal window.
pub fn render(ctx: &egui::Context, state: &mut AppState) {
    if !state.settings.show_settings {
        return;
    }

    egui::Window::new("⚙ Settings")
        .collapsible(false)
        .resizable(true)
        .default_width(400.0)
        .show(ctx, |ui| {
            ui.heading("Ollama Configuration");
            ui.add_space(8.0);

            // Endpoint
            ui.label(
                RichText::new("Endpoint URL")
                    .size(12.0)
                    .color(Color32::from_rgb(180, 180, 180)),
            );
            ui.add(
                TextEdit::singleline(&mut state.settings.ollama_endpoint)
                    .desired_width(300.0)
                    .hint_text("http://localhost:11434"),
            );

            ui.add_space(4.0);

            // Model
            ui.label(
                RichText::new("Model Name")
                    .size(12.0)
                    .color(Color32::from_rgb(180, 180, 180)),
            );
            ui.add(
                TextEdit::singleline(&mut state.settings.ollama_model)
                    .desired_width(300.0)
                    .hint_text("llama3.1:8b"),
            );

            ui.add_space(8.0);

            // Test connection
            ui.horizontal(|ui| {
                if ui.button("Test Connection").clicked() {
                    match chat::test_connection(&state.settings.ollama_endpoint) {
                        Ok(ok) => {
                            state.settings.connection_ok = Some(ok);
                        }
                        Err(_) => {
                            state.settings.connection_ok = Some(false);
                        }
                    }
                }

                match state.settings.connection_ok {
                    Some(true) => {
                        ui.label(
                            RichText::new("✓ Connected")
                                .color(Color32::from_rgb(100, 220, 100)),
                        );
                    }
                    Some(false) => {
                        ui.label(
                            RichText::new("✕ Not connected")
                                .color(Color32::from_rgb(255, 100, 100)),
                        );
                    }
                    None => {}
                }
            });

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);

            // Recent projects
            ui.heading("Recent Projects");
            ui.add_space(4.0);

            if state.project.recent.is_empty() {
                ui.label(
                    RichText::new("No recent projects")
                        .italics()
                        .size(12.0)
                        .color(Color32::from_rgb(120, 120, 120)),
                );
            } else {
                for p in &state.project.recent {
                    ui.label(
                        RichText::new(p.display().to_string())
                            .size(12.0)
                            .color(Color32::from_rgb(180, 180, 180)),
                    );
                }
            }

            ui.add_space(16.0);

            // Close button
            if ui.button("Close").clicked() {
                state.settings.show_settings = false;
            }
        });
}
