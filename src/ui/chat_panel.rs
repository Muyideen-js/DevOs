use egui::{self, Color32, RichText, TextEdit};
use std::path::PathBuf;

use crate::core::{chat, file_manager};
use crate::models::{ChatMessage, Role};
use crate::state::AppState;

/// Render the AI chat panel.
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label(
        RichText::new("AI ASSISTANT")
            .size(11.0)
            .strong()
            .color(Color32::from_rgb(140, 140, 140)),
    );
    ui.separator();

    // Context file selector
    render_context_selector(ui, state);

    ui.separator();

    // Chat messages
    let msg_height = ui.available_height() - 80.0;
    egui::ScrollArea::vertical()
        .max_height(msg_height)
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for msg in &state.chat.messages {
                let (prefix, color) = match msg.role {
                    Role::User => ("You", Color32::from_rgb(100, 180, 255)),
                    Role::Assistant => ("AI", Color32::from_rgb(100, 220, 150)),
                    Role::System => ("System", Color32::from_rgb(200, 200, 100)),
                };

                ui.label(
                    RichText::new(prefix)
                        .strong()
                        .size(11.0)
                        .color(color),
                );

                ui.label(
                    RichText::new(&msg.content)
                        .size(12.0)
                        .color(Color32::from_rgb(210, 210, 210)),
                );

                ui.add_space(8.0);
            }

            if state.chat.waiting_for_response {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(
                        RichText::new("Thinking...")
                            .italics()
                            .size(12.0)
                            .color(Color32::from_rgb(150, 150, 150)),
                    );
                });
            }
        });

    // Error display
    if let Some(ref err) = state.chat.last_error {
        ui.label(
            RichText::new(err)
                .size(11.0)
                .color(Color32::from_rgb(255, 120, 120)),
        );
    }

    // Input area
    ui.horizontal(|ui| {
        let response = ui.add(
            TextEdit::singleline(&mut state.chat.input)
                .desired_width(ui.available_width() - 50.0)
                .hint_text("Ask the AI assistant..."),
        );

        let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

        if (ui.button("Send").clicked() || enter_pressed) && !state.chat.waiting_for_response {
            send_message(state);
        }
    });
}

/// Render context file selector checkboxes.
fn render_context_selector(ui: &mut egui::Ui, state: &mut AppState) {
    egui::CollapsingHeader::new(
        RichText::new("📎 Context Files")
            .size(12.0)
            .color(Color32::from_rgb(180, 180, 180)),
    )
    .show(ui, |ui| {
        // Option to include terminal output
        ui.checkbox(
            &mut state.chat.include_terminal_output,
            RichText::new("Include last terminal output").size(11.0),
        );

        // List open editor tabs as context options
        let tab_paths: Vec<PathBuf> = state.editor.tabs.iter().map(|t| t.path.clone()).collect();

        for path in &tab_paths {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            // Find or create entry
            if let Some(entry) = state.chat.context_files.iter_mut().find(|(p, _)| p == path) {
                ui.checkbox(&mut entry.1, RichText::new(&name).size(11.0));
            } else {
                state.chat.context_files.push((path.clone(), false));
                let last = state.chat.context_files.last_mut().unwrap();
                ui.checkbox(&mut last.1, RichText::new(&name).size(11.0));
            }
        }

        // Clean up entries for closed tabs
        state
            .chat
            .context_files
            .retain(|(p, _)| tab_paths.contains(p));
    });
}

/// Send a user message and get AI response.
fn send_message(state: &mut AppState) {
    let input = state.chat.input.trim().to_string();
    if input.is_empty() {
        return;
    }

    // Add user message
    state.chat.messages.push(ChatMessage {
        role: Role::User,
        content: input.clone(),
    });

    state.chat.input.clear();

    // Gather context
    let mut context_files: Vec<(String, String)> = Vec::new();
    for (path, selected) in &state.chat.context_files {
        if *selected {
            if let Ok(content) = file_manager::read_file(path) {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                context_files.push((name, content));
            }
        }
    }

    let terminal_output = if state.chat.include_terminal_output {
        state
            .terminal
            .entries
            .last()
            .map(|e| e.output.clone())
    } else {
        None
    };

    // Try to send to Ollama (blocking in this MVP)
    let endpoint = state.settings.ollama_endpoint.clone();
    let model = state.settings.ollama_model.clone();

    let response = match chat::send_message_blocking(
        &endpoint,
        &model,
        &input,
        &context_files,
        terminal_output.as_deref(),
    ) {
        Ok(resp) => resp,
        Err(_) => chat::fallback_response(&input),
    };

    state.chat.messages.push(ChatMessage {
        role: Role::Assistant,
        content: response,
    });

    state.chat.last_error = None;
}
