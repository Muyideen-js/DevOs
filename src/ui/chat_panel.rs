use egui::{self, Color32, RichText, TextEdit, Vec2};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::core::{chat, file_manager};
use crate::models::{ChatMessage, Role};
use crate::state::AppState;

/// Render the AI chat panel.
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    // Header
    ui.label(
        RichText::new("AI ASSISTANT")
            .size(10.5)
            .strong()
            .color(Color32::from_rgb(140, 155, 175)),
    );
    ui.add_space(4.0);

    // Context file selector
    render_context_selector(ui, state);

    ui.add_space(4.0);

    // -- Poll async chat response --
    poll_response(state);
    if state.chat.waiting_for_response {
        ui.ctx().request_repaint();
    }

    // ── Message area ──
    let msg_height = ui.available_height() - 50.0;
    egui::ScrollArea::vertical()
        .max_height(msg_height)
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            if state.chat.messages.is_empty() && !state.chat.waiting_for_response {
                // Empty state
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("🤖")
                            .size(32.0)
                            .color(Color32::from_rgb(70, 75, 85)),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Ask me anything about your code")
                            .size(12.0)
                            .color(Color32::from_rgb(90, 95, 105)),
                    );
                });
            }

            for msg in &state.chat.messages {
                render_message_bubble(ui, msg);
            }

            if state.chat.waiting_for_response {
                ui.add_space(6.0);
                // AI thinking bubble
                egui::Frame::new()
                    .fill(Color32::from_rgb(30, 32, 40))
                    .corner_radius(10)
                    .inner_margin(egui::Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(
                                RichText::new("Thinking...")
                                    .italics()
                                    .size(12.0)
                                    .color(Color32::from_rgb(120, 130, 145)),
                            );
                        });
                    });
            }
        });

    // Error display
    if let Some(ref err) = state.chat.last_error {
        egui::Frame::new()
            .fill(Color32::from_rgb(55, 25, 25))
            .corner_radius(4)
            .inner_margin(egui::Margin::symmetric(8, 4))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(format!("⚠ {}", err))
                        .size(10.5)
                        .color(Color32::from_rgb(255, 140, 140)),
                );
            });
    }

    ui.add_space(4.0);

    // ── Input area ──
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);

        // Text input
        let input_width = ui.available_width() - 55.0;
        let response = ui.add_sized(
            Vec2::new(input_width.max(50.0), 30.0),
            TextEdit::singleline(&mut state.chat.input)
                .hint_text("Ask about your code...")
                .font(egui::TextStyle::Body)
                .text_color(Color32::from_rgb(210, 215, 225)),
        );

        // Send button
        let send_btn = egui::Button::new(
            RichText::new("➤")
                .size(16.0)
                .color(if state.chat.waiting_for_response {
                    Color32::from_rgb(60, 65, 75)
                } else {
                    Color32::from_rgb(90, 160, 255)
                }),
        )
        .fill(Color32::from_rgb(30, 50, 85))
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(50, 80, 130)))
        .corner_radius(6)
        .min_size(Vec2::new(36.0, 30.0));

        let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        let btn_clicked = ui.add_enabled(!state.chat.waiting_for_response, send_btn).clicked();

        if (btn_clicked || enter_pressed) && !state.chat.waiting_for_response {
            send_message(state);
            response.request_focus();
        }
    });
}

/// Render a single chat message as a bubble.
fn render_message_bubble(ui: &mut egui::Ui, msg: &ChatMessage) {
    let is_user = msg.role == Role::User;

    ui.add_space(4.0);

    if is_user {
        // User message — right-aligned blue bubble
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            let max_width = ui.available_width() * 0.85;
            ui.allocate_ui(Vec2::new(max_width, 0.0), |ui| {
                egui::Frame::new()
                    .fill(Color32::from_rgb(30, 55, 100))
                    .corner_radius(12)
                    .inner_margin(egui::Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("You")
                                .size(10.0)
                                .strong()
                                .color(Color32::from_rgb(130, 180, 255)),
                        );
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new(&msg.content)
                                .size(12.5)
                                .color(Color32::from_rgb(215, 225, 240)),
                        );
                    });
            });
        });
    } else {
        // AI message — left-aligned dark bubble
        let max_width = ui.available_width() * 0.9;
        ui.allocate_ui(Vec2::new(max_width, 0.0), |ui| {
            egui::Frame::new()
                .fill(Color32::from_rgb(28, 30, 38))
                .corner_radius(12)
                .inner_margin(egui::Margin::symmetric(12, 8))
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(42, 45, 55)))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("AI")
                            .size(10.0)
                            .strong()
                            .color(Color32::from_rgb(80, 200, 140)),
                    );
                    ui.add_space(2.0);

                    // Render content with monospace for code blocks
                    render_ai_content(ui, &msg.content);
                });
        });
    }

    ui.add_space(2.0);
}

/// Render AI message content, detecting code blocks.
fn render_ai_content(ui: &mut egui::Ui, content: &str) {
    let mut in_code_block = false;
    let mut code_buffer = String::new();

    for line in content.lines() {
        if line.starts_with("```") {
            if in_code_block {
                // End code block — render it
                egui::Frame::new()
                    .fill(Color32::from_rgb(18, 20, 26))
                    .corner_radius(6)
                    .inner_margin(egui::Margin::symmetric(8, 6))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(&code_buffer)
                                .monospace()
                                .size(11.5)
                                .color(Color32::from_rgb(200, 210, 225)),
                        );
                    });
                code_buffer.clear();
                in_code_block = false;
            } else {
                in_code_block = true;
            }
        } else if in_code_block {
            if !code_buffer.is_empty() {
                code_buffer.push('\n');
            }
            code_buffer.push_str(line);
        } else {
            // Normal text
            ui.label(
                RichText::new(line)
                    .size(12.5)
                    .color(Color32::from_rgb(200, 208, 220)),
            );
        }
    }

    // Handle unclosed code block
    if in_code_block && !code_buffer.is_empty() {
        egui::Frame::new()
            .fill(Color32::from_rgb(18, 20, 26))
            .corner_radius(6)
            .inner_margin(egui::Margin::symmetric(8, 6))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(&code_buffer)
                        .monospace()
                        .size(11.5)
                        .color(Color32::from_rgb(200, 210, 225)),
                );
            });
    }
}

/// Poll for async chat response.
fn poll_response(state: &mut AppState) {
    if !state.chat.waiting_for_response {
        return;
    }

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
            state.chat.last_error = Some(err.clone());
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
    }
}

/// Render context file selector checkboxes.
fn render_context_selector(ui: &mut egui::Ui, state: &mut AppState) {
    egui::CollapsingHeader::new(
        RichText::new("📎 Context")
            .size(11.0)
            .color(Color32::from_rgb(140, 150, 165)),
    )
    .show(ui, |ui| {
        ui.checkbox(
            &mut state.chat.include_terminal_output,
            RichText::new("Terminal output").size(10.5).color(Color32::from_rgb(160, 165, 175)),
        );

        let tab_paths: Vec<PathBuf> = state.editor.tabs.iter().map(|t| t.path.clone()).collect();

        for path in &tab_paths {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            if let Some(entry) = state.chat.context_files.iter_mut().find(|(p, _)| p == path) {
                ui.checkbox(&mut entry.1, RichText::new(&name).size(10.5).color(Color32::from_rgb(160, 165, 175)));
            } else {
                state.chat.context_files.push((path.clone(), false));
                let last = state.chat.context_files.last_mut().unwrap();
                ui.checkbox(&mut last.1, RichText::new(&name).size(10.5).color(Color32::from_rgb(160, 165, 175)));
            }
        }

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

    state.chat.messages.push(ChatMessage {
        role: Role::User,
        content: input.clone(),
    });

    state.chat.input.clear();
    state.chat.waiting_for_response = true;

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
        state.terminal.entries.last().map(|e| e.output.clone())
    } else {
        None
    };

    let endpoint = state.settings.ollama_endpoint.clone();
    let model = state.settings.ollama_model.clone();

    let result_arc: Arc<Mutex<Option<Result<String, String>>>> = Arc::new(Mutex::new(None));
    state.chat.pending_response = Some(result_arc.clone());

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

            let response = client
                .post(&url)
                .json(&body)
                .send()
                .map_err(|e| format!("{}", e))?;

            if !response.status().is_success() {
                let status = response.status();
                if let Ok(json) = response.json::<serde_json::Value>() {
                    if let Some(err_msg) = json.get("error").and_then(|e| e.as_str()) {
                        return Err(format!("{}", err_msg));
                    }
                }
                return Err(format!("Ollama returned status {}", status));
            }

            let json: serde_json::Value = response
                .json()
                .map_err(|e| format!("Invalid JSON: {}", e))?;

            json["response"]
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| "No 'response' in output".to_string())
        };

        let result = run_query();
        if let Ok(mut arc) = result_arc.lock() {
            *arc = Some(result);
        }
    });
}
