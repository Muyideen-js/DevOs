use egui::{self, Color32, RichText, TextEdit};
use std::sync::{Arc, Mutex};

use crate::core::terminal as term_core;
use crate::models::TerminalEntry;
use crate::state::AppState;

/// Render the terminal panel at the bottom.
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("TERMINAL")
                .size(10.5)
                .strong()
                .color(Color32::from_rgb(140, 155, 175)),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let clear_btn = egui::Button::new(
                RichText::new("Clear")
                    .size(10.5)
                    .color(Color32::from_rgb(140, 145, 155)),
            )
            .fill(Color32::TRANSPARENT)
            .stroke(egui::Stroke::NONE);
            if ui.add(clear_btn).clicked() {
                state.terminal.entries.clear();
            }
        });
    });

    ui.add_space(2.0);

    // -- Poll async terminal output --
    let mut finished = false;
    if let Some(ref is_running) = state.terminal.is_running {
        if let Ok(r) = is_running.try_lock() {
            finished = !*r;
        }
    }

    if let Some(ref output_buf) = state.terminal.running_output {
        if let Ok(mut buf) = output_buf.try_lock() {
            if !buf.is_empty() {
                if let Some(last_entry) = state.terminal.entries.last_mut() {
                    last_entry.output.push_str(&buf);
                    buf.clear();
                    state.terminal.scroll_to_bottom = true;
                }
            }
        }
    }

    if finished {
        if let Some(last_entry) = state.terminal.entries.last_mut() {
            last_entry.running = false;
        }
        state.terminal.is_running = None;
        state.terminal.running_output = None;
        state.terminal.running_child = None;
    }
    // ---------------------------------

    // Output area
    let output_height = ui.available_height() - 30.0;

    egui::ScrollArea::vertical()
        .max_height(output_height)
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for entry in &state.terminal.entries {
                // Command line
                ui.label(
                    RichText::new(format!("$ {}", entry.command))
                        .monospace()
                        .size(12.0)
                        .color(Color32::from_rgb(100, 200, 100)),
                );

                // Output
                let output_color = if entry.is_error {
                    Color32::from_rgb(255, 120, 120)
                } else {
                    Color32::from_rgb(200, 200, 200)
                };

                if !entry.output.is_empty() {
                    ui.label(
                        RichText::new(&entry.output)
                            .monospace()
                            .size(12.0)
                            .color(output_color),
                    );

                    let out_lower = entry.output.to_lowercase();
                    if out_lower.contains("error:") || out_lower.contains("error[") || out_lower.contains("panicked") {
                        ui.add_space(4.0);
                        let fix_btn = egui::Button::new(
                            RichText::new("🔧 Fix with AI").size(11.0).color(Color32::from_rgb(200, 220, 255)),
                        )
                        .fill(Color32::from_rgb(40, 60, 100))
                        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(60, 100, 160)))
                        .corner_radius(4);

                        if ui.add(fix_btn).clicked() {
                            state.right_tab = crate::models::RightTab::Chat;
                            state.chat.include_terminal_output = true;
                            state.chat.input = format!("Fix this error from the terminal for command `{}`:\n\n```\n{}\n```", entry.command, entry.output);
                            state.chat.auto_send = true;
                        }
                    }
                }

                if entry.running {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(RichText::new("Running...").italics().color(Color32::from_rgb(150, 150, 150)));
                    });
                }

                ui.add_space(4.0);
            }

            if state.terminal.scroll_to_bottom {
                ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                state.terminal.scroll_to_bottom = false;
            }
        });

    // Input area
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("$")
                .monospace()
                .size(13.0)
                .color(Color32::from_rgb(100, 200, 100)),
        );

        // Calculate available width capped properly
        let text_width = ui.available_width() - 120.0;
        let response = ui.add(
            TextEdit::singleline(&mut state.terminal.input)
                .font(egui::TextStyle::Monospace)
                .desired_width(if text_width > 50.0 { text_width } else { 50.0 })
                .hint_text("Enter command..."),
        );

        // Enter to run
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            execute_command(state);
            response.request_focus();
        }

        let has_running = state.terminal.running_child.is_some();

        let run_btn = egui::Button::new(
            RichText::new("▶ Run").size(11.0).color(Color32::from_rgb(80, 200, 130)),
        )
        .fill(Color32::from_rgb(25, 45, 35))
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(40, 70, 50)))
        .corner_radius(3);

        if ui.add_enabled(!has_running, run_btn).clicked() {
            execute_command(state);
            response.request_focus();
        }

        // Stop button
        let stop_btn = egui::Button::new(
            RichText::new("⬛ Stop").size(11.0).color(Color32::from_rgb(240, 100, 100)),
        )
        .fill(Color32::from_rgb(50, 25, 25))
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(70, 35, 35)))
        .corner_radius(3);

        if ui.add_enabled(has_running, stop_btn).clicked()
        {
            if let Some(ref child) = state.terminal.running_child {
                let _ = term_core::kill_process(child);
            }
            // Forcibly end
            if let Some(last_entry) = state.terminal.entries.last_mut() {
                last_entry.running = false;
                last_entry.output.push_str("\n[Process Terminated]\n");
            }
            state.terminal.running_child = None;
            state.terminal.is_running = None;
            state.terminal.running_output = None;
        }

        // Up/down history navigation
        if response.has_focus() {
            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                navigate_history(state, -1);
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                navigate_history(state, 1);
            }
        }
    });

    // Make sure we repaint continuously if a command is running so the output streams smoothly
    if state.terminal.running_child.is_some() {
        ui.ctx().request_repaint();
    }
}

/// Execute the current command input.
fn execute_command(state: &mut AppState) {
    let cmd = state.terminal.input.trim().to_string();
    if cmd.is_empty() {
        return;
    }

    crate::core::logger::info(&format!("Executing terminal command: {}", cmd));
    
    // Prevent starting multiple commands if one is already running
    if state.terminal.running_child.is_some() {
        return;
    }

    let cwd = state
        .project
        .root
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| ".".to_string());

    // Add to history
    state.terminal.command_history.push(cmd.clone());
    state.terminal.history_index = None;
    state.terminal.input.clear();

    // Push new entry in UI
    state.terminal.entries.push(TerminalEntry {
        command: cmd.clone(),
        output: String::new(),
        is_error: false,
        running: true,
    });
    state.terminal.scroll_to_bottom = true;

    // Run asynchronously
    let output_buf = Arc::new(Mutex::new(String::new()));
    let is_running = Arc::new(Mutex::new(true));

    let child = term_core::run_command_async(&cmd, &cwd, output_buf.clone(), is_running.clone());

    if let Some(child_arc) = child {
        state.terminal.running_child = Some(child_arc);
        state.terminal.running_output = Some(output_buf);
        state.terminal.is_running = Some(is_running);
    } else {
        // Failed to spawn
        if let Some(last_entry) = state.terminal.entries.last_mut() {
            last_entry.running = false;
            last_entry.is_error = true;
            last_entry.output = "Failed to spawn process".to_string();
        }
    }
}

/// Navigate command history with arrow keys.
fn navigate_history(state: &mut AppState, direction: i32) {
    if state.terminal.command_history.is_empty() {
        return;
    }

    let len = state.terminal.command_history.len();
    let new_index = match state.terminal.history_index {
        Some(idx) => {
            let new = idx as i32 + direction;
            if new < 0 {
                0usize
            } else if new >= len as i32 {
                return; // past end, do nothing
            } else {
                new as usize
            }
        }
        None => {
            if direction < 0 {
                len - 1
            } else {
                return;
            }
        }
    };

    state.terminal.history_index = Some(new_index);
    state.terminal.input = state.terminal.command_history[new_index].clone();
}
