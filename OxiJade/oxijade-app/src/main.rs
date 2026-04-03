#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod panels;
mod theme;

use eframe::NativeOptions;
use egui::ViewportBuilder;

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("OxiJade")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "OxiJade",
        options,
        Box::new(|_cc| Ok(Box::new(app::OxiJadeApp::default()))),
    )
}
