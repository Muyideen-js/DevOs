use egui::{self, Color32, RichText, TextEdit};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

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

    // -- Poll async chat response --
    if state.chat.waiting_for_response {
        let mut got_response = false;
        let mut response_text = None;
        let mut error_text = None;

        if let Some(ref pending) = state.chat.pending_response {
            if let Ok(mut result_opt) = pending.try_lock() {
                if let Some(result) = result_opt.take() {
                    got_response = true;
                    match result {
                        Ok(text) => response_text = Some(text),
                        Err(e) => error_text = Some(e),
                    }
                }
            }
        }

        if got_response {
            state.chat.waiting_for_response = false;
            state.chat.pending_response = None;

            if let Some(text) = response_text {
                state.chat.messages.push(ChatMessage {
                    role: Role::Assistant,
                    content: text,
                });
                state.chat.last_error = None;
            } else if let Some(err) = error_text {
                // Optionally push fallback
                state.chat.last_error = Some(err.clone());
                
                // Fetch the last user message to feed to the fallback
                let last_user_msg = state.chat.messages.iter()
                    .filter(|m| m.role == Role::User)
                    .last()
                    .map(|m| m.content.clone())
                    .unwrap_or_default();
                    
                state.chat.messages.push(ChatMessage {
                    role: Role::Assistant,
                    content: chat::fallback_response(&last_user_msg),
                });
            }
        } else {
            // Keep UI refreshing while waiting for the background thread
            ui.ctx().request_repaint();
        }
    }
    // ------------------------------

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
            RichText::new(format!("Error connecting to Ollama: {}", err))
                .size(11.0)
                .color(Color32::from_rgb(255, 120, 120)),
        );
    }

    // Input area
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
        let btn_clicked = ui.button("Send").clicked();

        let response = ui.add_sized(
            ui.available_size(),
            TextEdit::singleline(&mut state.chat.input).hint_text("Ask the AI assistant..."),
        );

        if (btn_clicked || (response.lost_focus() && enter_pressed)) && !state.chat.waiting_for_response {
            send_message(state);
            response.request_focus();
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

/// Send a user message processing asynchronously.
fn send_message(state: &mut AppState) {
    let input = state.chat.input.trim().to_string();
    if input.is_empty() || state.chat.waiting_for_response {
        return;
    }

    // Add user message to UI
    state.chat.messages.push(ChatMessage {
        role: Role::User,
        content: input.clone(),
    });

    state.chat.input.clear();
    state.chat.waiting_for_response = true;

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

    let endpoint = state.settings.ollama_endpoint.clone();
    let model = state.settings.ollama_model.clone();

    // Create the receiving channel
    let result_arc: Arc<Mutex<Option<Result<String, String>>>> = Arc::new(Mutex::new(None));
    state.chat.pending_response = Some(result_arc.clone());

    // Spawn thread to do blocking HTTP request
    thread::spawn(move || {
        let run_query = || -> Result<String, String> {
            let prompt = crate::core::chat::build_prompt(&input, &context_files, terminal_output.as_deref());

            let url = format!("{}/api/generate", endpoint.trim_end_matches('/'));

            let body = serde_json::json!({
                "model": model,
                "prompt": prompt,
                "system": "You are a coding assistant embedded in a developer IDE called DevOS. \
                           When the user asks you to modify code, output your changes as a unified diff. \
                           Be concise and helpful.",
                "stream": false,
            });

            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .map_err(|e| format!("HTTP client error: {}", e))?;

            // Note: Since this runs inside an async context spawned by thread::spawn, reqwest::blocking works perfectly without panic
            let response = client
                .post(&url)
                .json(&body)
                .send()
                .map_err(|e| format!("{}", e))?;

            if !response.status().is_success() {
                let status = response.status();
                if let Ok(json) = response.json::<serde_json::Value>() {
                    if let Some(err_msg) = json.get("error").and_then(|e| e.as_str()) {
                        return Err(format!("Error: {}", err_msg));
                    }
                }
                return Err(format!("Ollama returned status {}", status));
            }

            let json: serde_json::Value = response
                .json()
                .map_err(|e| format!("Invalid JSON response: {}", e))?;

            json["response"]
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| "No 'response' field in Ollama output".to_string())
        };

        let result = run_query();
        if let Ok(mut arc) = result_arc.lock() {
            *arc = Some(result);
        }
    });
}
