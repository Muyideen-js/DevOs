use egui::{self, Color32, RichText, TextEdit};

use crate::core::chat;
use crate::state::AppState;

/// Render the settings modal window.
pub fn render(ctx: &egui::Context, state: &mut AppState) {
    if !state.settings.show_settings {
        return;
    }

    egui::Window::new(RichText::new("⚙ Settings").size(15.0).color(Color32::from_rgb(200, 210, 230)))
        .collapsible(false)
        .resizable(true)
        .default_width(420.0)
        .frame(egui::Frame::new()
            .fill(Color32::from_rgb(28, 30, 38))
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(50, 55, 65)))
            .corner_radius(8)
            .inner_margin(egui::Margin::symmetric(16, 12)))
        .show(ctx, |ui| {
            // ── Ollama Section ──
            ui.label(
                RichText::new("AI Configuration")
                    .size(13.0)
                    .strong()
                    .color(Color32::from_rgb(100, 170, 255)),
            );
            ui.add_space(8.0);

            // Endpoint
            ui.label(
                RichText::new("Endpoint URL")
                    .size(11.0)
                    .color(Color32::from_rgb(140, 145, 155)),
            );
            ui.add(
                styled_input(&mut state.settings.ollama_endpoint, "http://localhost:11434"),
            );

            ui.add_space(6.0);

            // Model
            ui.label(
                RichText::new("Model Name")
                    .size(11.0)
                    .color(Color32::from_rgb(140, 145, 155)),
            );
            ui.add(
                styled_input(&mut state.settings.ollama_model, "llama3.1:8b"),
            );

            ui.add_space(10.0);

            // Test connection
            ui.horizontal(|ui| {
                let btn = egui::Button::new(
                    RichText::new("Test Connection")
                        .size(12.0)
                        .color(Color32::from_rgb(200, 215, 240)),
                )
                .fill(Color32::from_rgb(35, 55, 90))
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(60, 90, 140)))
                .corner_radius(4);

                if ui.add(btn).clicked() {
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
                            RichText::new("● Connected")
                                .size(12.0)
                                .color(Color32::from_rgb(80, 200, 120)),
                        );
                    }
                    Some(false) => {
                        ui.label(
                            RichText::new("● Disconnected")
                                .size(12.0)
                                .color(Color32::from_rgb(240, 90, 90)),
                        );
                    }
                    None => {}
                }
            });

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(10.0);

            // ── Recent Projects ──
            ui.label(
                RichText::new("Recent Projects")
                    .size(13.0)
                    .strong()
                    .color(Color32::from_rgb(100, 170, 255)),
            );
            ui.add_space(6.0);

            if state.project.recent.is_empty() {
                ui.label(
                    RichText::new("No recent projects")
                        .italics()
                        .size(11.0)
                        .color(Color32::from_rgb(90, 95, 105)),
                );
            } else {
                for p in &state.project.recent {
                    egui::Frame::new()
                        .fill(Color32::from_rgb(32, 34, 42))
                        .corner_radius(3)
                        .inner_margin(egui::Margin::symmetric(8, 3))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(p.display().to_string())
                                    .size(11.0)
                                    .monospace()
                                    .color(Color32::from_rgb(155, 160, 175)),
                            );
                        });
                    ui.add_space(2.0);
                }
            }

            ui.add_space(16.0);

            // Close button
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let btn = egui::Button::new(
                        RichText::new("Close")
                            .size(12.0)
                            .color(Color32::from_rgb(170, 175, 185)),
                    )
                    .fill(Color32::from_rgb(40, 42, 52))
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(55, 58, 68)))
                    .corner_radius(4);

                    if ui.add(btn).clicked() {
                        state.settings.show_settings = false;
                    }
                });
            });
        });
}

/// Create a styled text input.
fn styled_input<'a>(value: &'a mut String, hint: &'a str) -> TextEdit<'a> {
    TextEdit::singleline(value)
        .desired_width(320.0)
        .font(egui::TextStyle::Monospace)
        .hint_text(hint)
        .text_color(Color32::from_rgb(200, 205, 215))
}
