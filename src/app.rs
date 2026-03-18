use eframe::egui;

use crate::core::project;
use crate::models::RightTab;
use crate::state::AppState;
use crate::ui;

/// Main application struct implementing eframe::App.
pub struct DevOsApp {
    pub state: AppState,
}

impl DevOsApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load recent projects on startup
        let recent = project::load_recent_projects();

        let mut state = AppState::default();
        state.project.recent = recent;

        crate::core::logger::info("DevOS starting up...");

        // Optionally restore last project
        if let Some(last) = state.project.recent.first().cloned() {
            if last.exists() {
                let mut tree = crate::core::file_manager::read_dir_tree(&last, 8);
                crate::ui::file_explorer::expand_first_level(&mut tree);
                state.explorer.tree = tree;
                state.project.root = Some(last.clone());
                state.project.index = crate::core::indexer::build_project_index(&last);

                // Check git
                if let Some(repo) = crate::core::git::open_repo(&last) {
                    state.git.is_repo = true;
                    state.git.branch = crate::core::git::get_branch(&repo);
                    state.git.files = crate::core::git::get_status(&repo);
                }
            }
        }

        Self { state }
    }
}

impl eframe::App for DevOsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply dark theme with custom colors
        apply_theme(ctx);

        // Handle global keyboard shortcuts
        handle_shortcuts(ctx, &mut self.state);

        // Settings modal (rendered as a floating window)
        ui::settings::render(ctx, &mut self.state);

        // ── Top Bar ──
        egui::TopBottomPanel::top("top_bar")
            .min_height(40.0)
            .frame(egui::Frame::new()
                .fill(egui::Color32::from_rgb(20, 20, 25))
                .inner_margin(egui::Margin::symmetric(12, 6)))
            .show(ctx, |ui| {
                ui::top_bar::render(ui, &mut self.state);
            });

        // ── Bottom: Terminal Panel ──
        if self.state.terminal.show {
            egui::TopBottomPanel::bottom("terminal_panel")
                .min_height(120.0)
                .default_height(200.0)
                .resizable(true)
                .frame(egui::Frame::new()
                    .fill(egui::Color32::from_rgb(18, 18, 22))
                    .inner_margin(egui::Margin::symmetric(8, 6))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 40, 50))))
                .show(ctx, |ui| {
                    ui::terminal::render(ui, &mut self.state);
                });
        }

        // ── Left: File Explorer ──
        egui::SidePanel::left("file_explorer")
            .default_width(240.0)
            .min_width(160.0)
            .resizable(true)
            .frame(egui::Frame::new()
                .fill(egui::Color32::from_rgb(22, 22, 28))
                .inner_margin(egui::Margin::symmetric(6, 8))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(35, 35, 45))))
            .show(ctx, |ui| {
                ui::file_explorer::render(ui, &mut self.state);
            });

        // ── Right: Git + AI Chat (tabbed) ──
        egui::SidePanel::right("right_panel")
            .default_width(300.0)
            .min_width(220.0)
            .resizable(true)
            .frame(egui::Frame::new()
                .fill(egui::Color32::from_rgb(22, 22, 28))
                .inner_margin(egui::Margin::symmetric(8, 8))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(35, 35, 45))))
            .show(ctx, |ui| {
                // Tabs for Git / Chat / Patches
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::Vec2::new(2.0, 0.0);

                    let tabs = [
                        (RightTab::Git, "⎇ Git"),
                        (RightTab::Chat, "🤖 Chat"),
                        (RightTab::Patches, "🩹 Patches"),
                    ];

                    for (tab, label) in tabs {
                        let selected = self.state.right_tab == tab;
                        let text = if selected {
                            egui::RichText::new(label).size(12.5).strong()
                                .color(egui::Color32::from_rgb(100, 180, 255))
                        } else {
                            egui::RichText::new(label).size(12.5)
                                .color(egui::Color32::from_rgb(140, 140, 150))
                        };

                        if ui.selectable_label(selected, text).clicked() {
                            self.state.right_tab = tab;
                        }
                    }
                });

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                match self.state.right_tab {
                    RightTab::Git => ui::git_panel::render(ui, &mut self.state),
                    RightTab::Chat => ui::chat_panel::render(ui, &mut self.state),
                    RightTab::Patches => ui::patch_view::render(ui, &mut self.state),
                }
            });

        // ── Center: Editor ──
        egui::CentralPanel::default()
            .frame(egui::Frame::new()
                .fill(egui::Color32::from_rgb(28, 28, 34))
                .inner_margin(egui::Margin::symmetric(4, 4)))
            .show(ctx, |ui| {
                ui::editor::render(ui, &mut self.state);
            });

        // ── Overlays: Command Palette ──
        ui::command_palette::render(ctx, &mut self.state);
    }
}

/// Apply a premium dark theme inspired by modern IDEs.
fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    let mut visuals = egui::Visuals::dark();
    visuals.dark_mode = true;
    visuals.override_text_color = Some(egui::Color32::from_rgb(200, 205, 215));
    visuals.panel_fill = egui::Color32::from_rgb(25, 25, 30);
    visuals.window_fill = egui::Color32::from_rgb(28, 28, 34);
    visuals.faint_bg_color = egui::Color32::from_rgb(35, 35, 42);
    visuals.extreme_bg_color = egui::Color32::from_rgb(15, 15, 18);

    // Widgets
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(38, 38, 46);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 50, 60));
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(50, 50, 60);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 130, 200));
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(60, 70, 90);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(90, 150, 230));

    // Selection
    visuals.selection.bg_fill = egui::Color32::from_rgb(40, 70, 120);
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(90, 160, 255));

    // Separators
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 40, 50));

    // Window
    visuals.window_shadow = egui::epaint::Shadow::NONE;
    visuals.popup_shadow = egui::epaint::Shadow::NONE;

    style.visuals = visuals;
    style.spacing.item_spacing = egui::Vec2::new(6.0, 3.0);
    style.spacing.button_padding = egui::Vec2::new(8.0, 3.0);

    ctx.set_style(style);
}

/// Handle global keyboard shortcuts.
fn handle_shortcuts(ctx: &egui::Context, state: &mut AppState) {
    ctx.input(|i| {
        // Ctrl+S → Save
        if i.modifiers.ctrl && i.key_pressed(egui::Key::S) {
            if let Some(msg) = ui::editor::handle_save(state) {
                // Store status message for display
                state.status_message = Some(msg);
            }
        }

        // Ctrl+F → Toggle search
        if i.modifiers.ctrl && i.key_pressed(egui::Key::F) {
            state.editor.show_search = !state.editor.show_search;
        }

        // Ctrl+` → Toggle terminal
        if i.modifiers.ctrl && i.key_pressed(egui::Key::Backtick) {
            state.terminal.show = !state.terminal.show;
        }

        // Ctrl+Shift+P → Command Palette
        if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::P) {
            state.command_palette.show = true;
            state.command_palette.query.clear();
        }
    });
}
