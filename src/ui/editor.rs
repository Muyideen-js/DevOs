use egui::{self, Color32, RichText, TextEdit, Vec2};
use egui_extras::syntax_highlighting::{self, CodeTheme};

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

                        let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                            let mut job = syntax_highlighting::highlight(
                                ui.ctx(),
                                ui.style(),
                                &theme,
                                text,
                                &lang_str,
                            );
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

                        ui.add(text_edit);
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
