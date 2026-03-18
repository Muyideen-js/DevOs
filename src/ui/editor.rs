use egui::{self, Color32, RichText, TextEdit, Vec2};
use egui_extras::syntax_highlighting::{self, CodeTheme};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::core::file_manager;
use crate::state::AppState;

/// Get the syntax language string from a filename.
fn lang_from_filename(name: &str) -> &'static str {
    if let Some(ext) = name.rsplit('.').next() {
        match ext.to_lowercase().as_str() {
            "rs"  => "rs",
            "py"  => "py",
            "js" | "jsx" | "mjs" => "js",
            "ts" | "tsx" => "ts",
            "html" | "htm" => "html",
            "css" | "scss" | "sass" => "css",
            "json" => "json",
            "toml" => "toml",
            "yaml" | "yml" => "yaml",
            "md"  => "md",
            "sh" | "bash" | "zsh" => "sh",
            "sql" => "sql",
            "c" | "h" => "c",
            "cpp" | "cxx" | "hpp" => "cpp",
            "go" => "go",
            "xml" => "xml",
            _ => "txt",
        }
    } else {
        "txt"
    }
}

/// Render the tabbed editor panel in the center.
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    if state.editor.tabs.is_empty() {
        render_welcome(ui);
        return;
    }

    // ── Tab bar ──
    let mut tab_to_close: Option<usize> = None;
    let mut tab_to_activate: Option<usize> = None;

    egui::Frame::new()
        .fill(Color32::from_rgb(22, 23, 28))
        .inner_margin(egui::Margin::symmetric(4, 0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(1.0, 0.0);
                for (i, tab) in state.editor.tabs.iter().enumerate() {
                    let is_active = state.editor.active_tab == Some(i);
                    let title = tab.title();

                    let bg = if is_active {
                        Color32::from_rgb(38, 40, 48)
                    } else {
                        Color32::from_rgb(26, 27, 32)
                    };

                    let text_color = if tab.is_modified() {
                        Color32::from_rgb(255, 200, 100)
                    } else if is_active {
                        Color32::from_rgb(220, 225, 235)
                    } else {
                        Color32::from_rgb(120, 125, 135)
                    };

                    let icon_color = file_icon_color(&tab.label);

                    let frame = egui::Frame::new()
                        .fill(bg)
                        .inner_margin(egui::Margin::symmetric(10, 5))
                        .corner_radius(egui::CornerRadius {
                            nw: 6, ne: 6, sw: 0, se: 0,
                        });

                    frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);

                            // Colored dot
                            ui.label(RichText::new("●").size(10.0).color(icon_color));

                            // Tab label
                            if ui
                                .selectable_label(false, RichText::new(&title).color(text_color).size(12.0))
                                .clicked()
                            {
                                tab_to_activate = Some(i);
                            }

                            // Close button
                            let close_color = if is_active {
                                Color32::from_rgb(140, 145, 155)
                            } else {
                                Color32::from_rgb(80, 82, 90)
                            };
                            if ui
                                .small_button(RichText::new("✕").size(9.0).color(close_color))
                                .clicked()
                            {
                                tab_to_close = Some(i);
                            }
                        });
                    });
                }
            });
        });

    // Handle tab actions
    if let Some(idx) = tab_to_activate {
        state.editor.active_tab = Some(idx);
    }
    if let Some(idx) = tab_to_close {
        state.editor.close_tab(idx);
    }

    // Active tab accent line
    if state.editor.active_tab.is_some() {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 2.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(70, 130, 220));
    }

    // ── Editor content ──
    if let Some(active) = state.editor.active_tab {
        if active < state.editor.tabs.len() {
            // Search bar (Ctrl+F)
            if state.editor.show_search {
                egui::Frame::new()
                    .fill(Color32::from_rgb(32, 34, 42))
                    .inner_margin(egui::Margin::symmetric(10, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("🔍")
                                    .size(13.0)
                                    .color(Color32::from_rgb(100, 160, 240)),
                            );
                            let search_width = (ui.available_width() - 30.0).max(50.0);
                            ui.add_sized(
                                Vec2::new(search_width, 22.0),
                                TextEdit::singleline(&mut state.editor.search_query)
                                    .font(egui::TextStyle::Monospace)
                                    .hint_text("Find in file..."),
                            );
                            if ui
                                .small_button(RichText::new("✕").size(11.0).color(Color32::from_rgb(150, 150, 160)))
                                .clicked()
                            {
                                state.editor.show_search = false;
                                state.editor.search_query.clear();
                            }
                        });
                    });
            }

            let lang = lang_from_filename(&state.editor.tabs[active].label);
            let theme = CodeTheme::dark(14.0);

            // ── Code area with line numbers + syntax highlighting ──
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(0.0, 0.0);

                        let tab = &state.editor.tabs[active];
                        let line_count = tab.content.lines().count().max(1);

                        // Line numbers gutter
                        egui::Frame::new()
                            .fill(Color32::from_rgb(20, 21, 26))
                            .inner_margin(egui::Margin { left: 6, right: 10, top: 4, bottom: 4 })
                            .show(ui, |ui| {
                                let mut line_nums = String::new();
                                for i in 1..=line_count {
                                    if i > 1 { line_nums.push('\n'); }
                                    line_nums.push_str(&format!("{:>4}", i));
                                }
                                ui.label(
                                    RichText::new(line_nums)
                                        .monospace()
                                        .size(14.0)
                                        .color(Color32::from_rgb(65, 70, 82)),
                                );
                            });

                        // Code editor with syntax highlighting
                        let tab = &mut state.editor.tabs[active];
                        let lang_str = lang.to_string();

                        let ghost_text_clone = tab.ghost_text.clone();
                        let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                            let mut job = syntax_highlighting::highlight(
                                ui.ctx(),
                                ui.style(),
                                &theme,
                                text,
                                &lang_str,
                            );
                            
                            if let Some(ghost) = &ghost_text_clone {
                                job.append(
                                    ghost,
                                    0.0,
                                    egui::TextFormat {
                                        color: Color32::from_rgb(100, 105, 115),
                                        font_id: egui::FontId::monospace(14.0),
                                        italics: true,
                                        ..Default::default()
                                    },
                                );
                            }

                            job.wrap.max_width = wrap_width;
                            ui.fonts(|f| f.layout_job(job))
                        };

                        let text_edit = TextEdit::multiline(&mut tab.content)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .desired_width(f32::INFINITY)
                            .desired_rows(line_count.max(30))
                            .lock_focus(true)
                            .layouter(&mut layouter);

                        let response = ui.add(text_edit);
                        
                        // Handle Autocomplete interactions
                        if response.changed() {
                            tab.last_content_change = Some(Instant::now());
                            tab.ghost_text = None;
                            tab.pending_completion = None;
                        }

                        // Accept ghost text on Tab
                        if let Some(ghost) = &tab.ghost_text {
                            if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Tab)) {
                                // Strip the tab character inserted by TextEdit
                                if tab.content.ends_with('\t') {
                                    tab.content.pop();
                                }
                                tab.content.push_str(ghost);
                                tab.ghost_text = None;
                                tab.pending_completion = None;
                            }
                        }

                        // Trigger Autocomplete
                        if !response.changed() && tab.ghost_text.is_none() && tab.pending_completion.is_none() {
                            if let Some(last_change) = tab.last_content_change {
                                if last_change.elapsed() > Duration::from_millis(1500) && !tab.content.trim().is_empty() {
                                    // Spawn request to Ollama
                                    let endpoint = state.settings.ollama_endpoint.clone();
                                    let model = state.settings.ollama_model.clone();
                                    
                                    // Prepare prompt - give it the last 1500 chars 
                                    let text_len = tab.content.len();
                                    let start = text_len.saturating_sub(1500);
                                    let prompt = format!("Complete the following code. Output only the exact next few lines of code that should follow. Do not include markdown blocks or explanation. Code:\n{}", &tab.content[start..]);

                                    let result_arc: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
                                    tab.pending_completion = Some(result_arc.clone());
                                    let ctx_clone = ui.ctx().clone();

                                    thread::spawn(move || {
                                        let url = format!("{}/api/generate", endpoint.trim_end_matches('/'));
                                        let body = serde_json::json!({
                                            "model": model,
                                            "prompt": prompt,
                                            "stream": false,
                                            "options": {
                                                "temperature": 0.2,
                                                "num_predict": 50,
                                                "stop": ["\n\n"]
                                            }
                                        });

                                        let client = reqwest::blocking::Client::builder()
                                            .timeout(Duration::from_secs(5))
                                            .build().unwrap_or_default();

                                        if let Ok(resp) = client.post(&url).json(&body).send() {
                                            if let Ok(json) = resp.json::<serde_json::Value>() {
                                                if let Some(text) = json["response"].as_str() {
                                                    let clean = text.trim_start_matches('\n').replace("```", "");
                                                    if !clean.trim().is_empty() {
                                                        if let Ok(mut arc) = result_arc.lock() {
                                                            *arc = Some(clean);
                                                            ctx_clone.request_repaint();
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    });
                                }
                            }
                        }

                        // Poll Autocomplete Result
                        if let Some(pending) = &tab.pending_completion {
                            if let Ok(mut arc) = pending.try_lock() {
                                if let Some(text) = arc.take() {
                                    tab.ghost_text = Some(text);
                                }
                            }
                        }
                        
                        // Keep track if user clicked explain
                        let mut explain_clicked = false;
                        response.context_menu(|ui| {
                            if ui.button(RichText::new("🧠 Explain File with AI").size(12.0)).clicked() {
                                explain_clicked = true;
                                ui.close_menu();
                            }
                        });

                        if explain_clicked {
                            state.right_tab = crate::models::RightTab::Chat;
                            state.chat.input = format!("Please explain the code in `{}`. What does it do and how does it work?", tab.label);
                            // Ensure the file is selected in context
                            let path = tab.path.clone();
                            if !state.chat.context_files.iter().any(|(p, _)| p == &path) {
                                state.chat.context_files.push((path.clone(), true));
                            } else {
                                if let Some(entry) = state.chat.context_files.iter_mut().find(|(p, _)| p == &path) {
                                    entry.1 = true;
                                }
                            }
                            state.chat.auto_send = true;
                        }
                    });
                });
        }
    }
}

/// Render the welcome screen when no files are open.
fn render_welcome(ui: &mut egui::Ui) {
    ui.add_space(ui.available_height() * 0.22);
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new("⚡")
                .size(56.0)
                .color(Color32::from_rgb(80, 140, 230)),
        );
        ui.add_space(12.0);
        ui.label(
            RichText::new("DevOS")
                .size(28.0)
                .strong()
                .color(Color32::from_rgb(100, 170, 255)),
        );
        ui.add_space(6.0);
        ui.label(
            RichText::new("Your Local AI Developer Workspace")
                .size(14.0)
                .color(Color32::from_rgb(110, 115, 125)),
        );
        ui.add_space(24.0);

        let hints = [
            ("Ctrl+O", "Open Project"),
            ("Ctrl+S", "Save File"),
            ("Ctrl+F", "Find in File"),
            ("Ctrl+`", "Toggle Terminal"),
            ("Ctrl+Shift+P", "Command Palette"),
        ];
        for (key, desc) in hints {
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() * 0.28);
                egui::Frame::new()
                    .fill(Color32::from_rgb(35, 38, 48))
                    .corner_radius(4)
                    .inner_margin(egui::Margin::symmetric(8, 3))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(key)
                                .monospace()
                                .size(11.5)
                                .color(Color32::from_rgb(130, 170, 245)),
                        );
                    });
                ui.add_space(8.0);
                ui.label(
                    RichText::new(desc)
                        .size(12.5)
                        .color(Color32::from_rgb(120, 125, 138)),
                );
            });
            ui.add_space(2.0);
        }
    });
}

/// Handle Ctrl+S to save the active file.
pub fn handle_save(state: &mut AppState) -> Option<String> {
    if let Some(active) = state.editor.active_tab {
        if active < state.editor.tabs.len() {
            let tab = &mut state.editor.tabs[active];
            match file_manager::write_file(&tab.path, &tab.content) {
                Ok(()) => {
                    tab.original = tab.content.clone();
                    return Some(format!("✓ Saved {}", tab.label));
                }
                Err(e) => {
                    return Some(format!("✕ Save failed: {}", e));
                }
            }
        }
    }
    None
}

// ── File type color for tab dots ──

fn file_icon_color(name: &str) -> Color32 {
    if let Some(ext) = name.rsplit('.').next() {
        match ext.to_lowercase().as_str() {
            "rs"  => Color32::from_rgb(222, 143, 60),
            "py"  => Color32::from_rgb(55, 165, 85),
            "js" | "jsx" => Color32::from_rgb(245, 215, 75),
            "ts" | "tsx" => Color32::from_rgb(50, 130, 225),
            "html" | "htm" => Color32::from_rgb(235, 105, 55),
            "css" | "scss" => Color32::from_rgb(85, 155, 235),
            "json" => Color32::from_rgb(185, 185, 80),
            "toml" | "yaml" | "yml" => Color32::from_rgb(160, 130, 200),
            "md" | "txt" => Color32::from_rgb(120, 170, 225),
            _ => Color32::from_rgb(130, 135, 145),
        }
    } else {
        Color32::from_rgb(130, 135, 145)
    }
}
