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
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Load recent projects on startup
        let recent = project::load_recent_projects();

        let mut state = AppState::default();
        state.project.recent = recent;

        // Optionally restore last project
        if let Some(last) = state.project.recent.first().cloned() {
            if last.exists() {
                let tree = crate::core::file_manager::read_dir_tree(&last, 8);
                state.explorer.tree = tree;
                state.project.root = Some(last.clone());

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
            .min_height(36.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui::top_bar::render(ui, &mut self.state);
                ui.add_space(4.0);
            });

        // ── Bottom: Terminal Panel ──
        if self.state.terminal.show {
            egui::TopBottomPanel::bottom("terminal_panel")
                .min_height(150.0)
                .default_height(200.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui::terminal::render(ui, &mut self.state);
                });
        }

        // ── Left: File Explorer ──
        egui::SidePanel::left("file_explorer")
            .default_width(220.0)
            .min_width(150.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui::file_explorer::render(ui, &mut self.state);
            });

        // ── Right: Git + AI Chat (tabbed) ──
        egui::SidePanel::right("right_panel")
            .default_width(300.0)
            .min_width(200.0)
            .resizable(true)
            .show(ctx, |ui| {
                // Tabs for Git / Chat
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.state.right_tab,
                        RightTab::Git,
                        egui::RichText::new("⎇ Git").size(13.0),
                    );
                    ui.selectable_value(
                        &mut self.state.right_tab,
                        RightTab::Chat,
                        egui::RichText::new("🤖 AI Chat").size(13.0),
                    );
                    ui.selectable_value(
                        &mut self.state.right_tab,
                        RightTab::Patches,
                        egui::RichText::new("🩹 Patches").size(13.0),
                    );
                });
                ui.separator();

                match self.state.right_tab {
                    RightTab::Git => ui::git_panel::render(ui, &mut self.state),
                    RightTab::Chat => ui::chat_panel::render(ui, &mut self.state),
                    RightTab::Patches => ui::patch_view::render(ui, &mut self.state),
                }
            });

        // ── Center: Editor ──
        egui::CentralPanel::default().show(ctx, |ui| {
            ui::editor::render(ui, &mut self.state);
        });
    }
}


/// Apply a custom dark theme to the egui context.
fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Dark color scheme
    let visuals = egui::Visuals {
        dark_mode: true,
        override_text_color: Some(egui::Color32::from_rgb(210, 210, 210)),
        panel_fill: egui::Color32::from_rgb(25, 25, 30),
        window_fill: egui::Color32::from_rgb(30, 30, 35),
        ..egui::Visuals::dark()
    };

    style.visuals = visuals;
    style.spacing.item_spacing = egui::Vec2::new(6.0, 4.0);
    ctx.set_style(style);
}

/// Handle global keyboard shortcuts.
fn handle_shortcuts(ctx: &egui::Context, state: &mut AppState) {
    ctx.input(|i| {
        // Ctrl+S → Save
        if i.modifiers.ctrl && i.key_pressed(egui::Key::S) {
            if let Some(msg) = ui::editor::handle_save(state) {
                eprintln!("{}", msg); // MVP: print to console
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
    });
}
