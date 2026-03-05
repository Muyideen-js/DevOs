mod app;
mod core;
mod models;
mod state;
mod ui;

use app::DevOsApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    // Configure the native window
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("DevOS — Developer Workspace")
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "DevOS",
        options,
        Box::new(|cc| Ok(Box::new(DevOsApp::new(cc)))),
    )
}

