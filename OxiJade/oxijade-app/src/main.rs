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
        Box::new(|cc| {
            // Load a system CJK font so Chinese characters render correctly
            let mut fonts = egui::FontDefinitions::default();
            for path in &[
                "C:/Windows/Fonts/msyh.ttc",  // Microsoft YaHei
                "C:/Windows/Fonts/msyh.ttf",
                "C:/Windows/Fonts/simsun.ttc", // SimSun fallback
            ] {
                if let Ok(data) = std::fs::read(path) {
                    fonts
                        .font_data
                        .insert("cjk".to_owned(), egui::FontData::from_owned(data));
                    // Add CJK font as a fallback after the default font
                    fonts
                        .families
                        .entry(egui::FontFamily::Proportional)
                        .or_default()
                        .push("cjk".to_owned());
                    fonts
                        .families
                        .entry(egui::FontFamily::Monospace)
                        .or_default()
                        .push("cjk".to_owned());
                    break;
                }
            }
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(app::OxiJadeApp::default()))
        }),
    )
}
