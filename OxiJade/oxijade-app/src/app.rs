use crate::theme::apply_theme;
use egui::Context;
use oxijade_config::{load_profiles, ProfileStore};

pub struct OxiJadeApp {
    pub active_tab: Option<String>,
    pub open_tabs: Vec<String>,
    pub sidebar_width: f32,
    pub profiles: ProfileStore,
}

impl Default for OxiJadeApp {
    fn default() -> Self {
        let mut profiles = load_profiles();
        if profiles.groups.is_empty() {
            use oxijade_config::{LocalProfile, SessionGroup, SessionProfile};
            profiles.groups.push(SessionGroup {
                name: "本地".to_string(),
                sessions: vec![SessionProfile::Local(LocalProfile {
                    id: "local-powershell".to_string(),
                    name: "PowerShell".to_string(),
                    shell: "powershell.exe".to_string(),
                })],
            });
        }
        Self {
            active_tab: None,
            open_tabs: Vec::new(),
            sidebar_width: 200.0,
            profiles,
        }
    }
}

impl eframe::App for OxiJadeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        apply_theme(ctx);

        egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
            crate::panels::tab_bar::show(ui, self);
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                if let Some(tab_id) = &self.active_tab {
                    ui.label(
                        egui::RichText::new(format!("● {}", tab_id))
                            .color(crate::theme::Theme::ACCENT_LOCAL)
                            .size(11.0),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("空闲")
                            .color(crate::theme::Theme::TEXT_MUTED)
                            .size(11.0),
                    );
                }
            });
        });

        egui::SidePanel::left("sidebar")
            .default_width(self.sidebar_width)
            .width_range(150.0..=300.0)
            .show(ctx, |ui| {
                crate::panels::sidebar::show(ui, self);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            crate::panels::terminal::show(ui, self);
        });
    }
}
