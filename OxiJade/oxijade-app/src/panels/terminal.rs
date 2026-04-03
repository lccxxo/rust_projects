pub fn show(ui: &mut egui::Ui, _app: &mut crate::app::OxiJadeApp) {
    ui.centered_and_justified(|ui| {
        ui.label(
            egui::RichText::new("选择或创建一个会话")
                .color(crate::theme::Theme::TEXT_MUTED)
                .size(14.0),
        );
    });
}
