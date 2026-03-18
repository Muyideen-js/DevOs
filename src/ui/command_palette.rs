use egui::{self, Color32, RichText, TextEdit, Vec2};

use crate::state::AppState;

#[derive(Clone)]
struct Command {
    title: String,
    subtitle: String,
    action: fn(&mut AppState),
}

/// Helper to declare available commands
fn get_commands() -> Vec<Command> {
    vec![
        Command {
            title: "Open File / Project".to_string(),
            subtitle: "Browse your file system to open a directory or file".to_string(),
            action: |state| {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    state.project.root = Some(path.clone());
                    state.project.index = crate::core::indexer::build_project_index(&path);
                    state.explorer.tree.clear();
                    state.editor.tabs.clear();
                    state.editor.active_tab = None;
                    if !state.project.recent.contains(&path) {
                        state.project.recent.push(path);
                    }
                }
            },
        },
        Command {
            title: "Toggle Terminal".to_string(),
            subtitle: "Show or hide the bottom terminal panel".to_string(),
            action: |state| state.terminal.show = !state.terminal.show,
        },
        Command {
            title: "Search in File".to_string(),
            subtitle: "Press Ctrl+F to open the search bar in the active editor".to_string(),
            action: |state| state.editor.show_search = true,
        },
        Command {
            title: "Ask AI Assistant".to_string(),
            subtitle: "Focus the AI chat panel".to_string(),
            action: |_| { /* handled separately by UI focus if needed, but we can just toggle right tab */ },
        },
        Command {
            title: "Settings".to_string(),
            subtitle: "Open the DevOS settings modal".to_string(),
            action: |state| state.settings.show_settings = true,
        },
    ]
}

pub fn render(ctx: &egui::Context, state: &mut AppState) {
    if !state.command_palette.show {
        return;
    }

    let commands = get_commands();
    let query = state.command_palette.query.to_lowercase();
    let filtered: Vec<&Command> = if query.is_empty() {
        commands.iter().collect()
    } else {
        commands.iter().filter(|c| c.title.to_lowercase().contains(&query)).collect()
    };

    // Keep selected index in bounds
    if filtered.is_empty() {
        state.command_palette.selected_index = 0;
    } else {
        state.command_palette.selected_index = state.command_palette.selected_index.min(filtered.len() - 1);
    }

    // Modal overlay
    egui::Window::new("Command Palette")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .fixed_size(Vec2::new(500.0, 300.0))
        .pivot(egui::Align2::CENTER_TOP)
        .fixed_pos(egui::pos2(ctx.screen_rect().center().x, ctx.screen_rect().top() + 50.0))
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(Color32::from_rgb(25, 27, 33))
                .corner_radius(8)
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(60, 65, 75)))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 8],
                    blur: 16,
                    spread: 0,
                    color: Color32::from_black_alpha(150),
                }),
        )
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(0.0, 0.0);

            // ── Input Field ──
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(12, 10))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🔍").size(16.0).color(Color32::from_rgb(100, 150, 240)));
                        ui.add_space(8.0);
                        
                        let response = ui.add_sized(
                            Vec2::new(ui.available_width(), 24.0),
                            TextEdit::singleline(&mut state.command_palette.query)
                                .hint_text("Search commands...")
                                .font(egui::TextStyle::Body)
                                .text_color(Color32::from_rgb(220, 225, 235))
                        );

                        // Auto-focus the input
                        response.request_focus();

                        // Handle keyboard
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                            if !filtered.is_empty() {
                                state.command_palette.selected_index = (state.command_palette.selected_index + 1) % filtered.len();
                            }
                        }
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                            if !filtered.is_empty() {
                                if state.command_palette.selected_index == 0 {
                                    state.command_palette.selected_index = filtered.len() - 1;
                                } else {
                                    state.command_palette.selected_index -= 1;
                                }
                            }
                        }
                        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            if let Some(cmd) = filtered.get(state.command_palette.selected_index) {
                                (cmd.action)(state);
                                state.command_palette.show = false;
                                state.command_palette.query.clear();
                            }
                        }
                        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            state.command_palette.show = false;
                            state.command_palette.query.clear();
                        }
                    });
                });

            ui.add_space(2.0);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(45, 50, 60));

            // ── Results List ──
            egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                ui.add_space(4.0);
                for (i, cmd) in filtered.iter().enumerate() {
                    let is_selected = i == state.command_palette.selected_index;
                    let bg = if is_selected {
                        Color32::from_rgb(40, 60, 100)
                    } else {
                        Color32::TRANSPARENT
                    };

                    egui::Frame::new()
                        .fill(bg)
                        .inner_margin(egui::Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            let response = ui.allocate_response(ui.available_size(), egui::Sense::click());
                            
                            if response.hovered() && !is_selected {
                                state.command_palette.selected_index = i;
                            }

                            if response.clicked() {
                                (cmd.action)(state);
                                state.command_palette.show = false;
                                state.command_palette.query.clear();
                            }

                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(&cmd.title)
                                        .size(13.0)
                                        .color(if is_selected { Color32::from_rgb(240, 245, 255) } else { Color32::from_rgb(200, 205, 215) })
                                );
                                ui.add_space(1.0);
                                ui.label(
                                    RichText::new(&cmd.subtitle)
                                        .size(11.0)
                                        .color(if is_selected { Color32::from_rgb(160, 180, 220) } else { Color32::from_rgb(120, 125, 135) })
                                );
                            });
                        });
                }

                if filtered.is_empty() {
                    ui.add_space(20.0);
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("No matching commands").color(Color32::from_rgb(100, 105, 115)));
                    });
                    ui.add_space(20.0);
                }
            });
        });
}
