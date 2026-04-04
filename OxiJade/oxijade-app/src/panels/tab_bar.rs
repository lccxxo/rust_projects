// oxijade-app/src/panels/tab_bar.rs
use crate::app::OxiJadeApp;
use crate::theme::Theme;
use egui::{Color32, RichText, Sense, Ui};
use oxijade_config::SessionProfile;

pub fn show(ui: &mut Ui, app: &mut OxiJadeApp) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        ui.label(RichText::new("⬡").color(Theme::ACCENT_SSH).size(16.0));
        ui.label(
            RichText::new("OxiJade")
                .color(Theme::TEXT_PRIMARY)
                .size(13.0)
                .strong(),
        );
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(4.0);

        let tabs = app.open_tabs.clone();
        let mut to_close: Option<String> = None;

        for tab_id in &tabs {
            let is_active = app.active_tab.as_deref() == Some(tab_id.as_str());
            let tab_name =
                find_profile_name(&app.profiles.groups, tab_id).unwrap_or_else(|| tab_id.clone());
            let accent = find_profile_accent(&app.profiles.groups, tab_id);

            let bg = if is_active {
                Theme::BG_PANEL
            } else {
                Color32::TRANSPARENT
            };

            let tab_response = egui::Frame::none()
                .fill(bg)
                .rounding(egui::Rounding {
                    nw: 4.0,
                    ne: 4.0,
                    sw: 0.0,
                    se: 0.0,
                })
                .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                .show(ui, |ui| {
                    if is_active {
                        ui.painter().hline(
                            ui.max_rect().x_range(),
                            ui.max_rect().top(),
                            egui::Stroke::new(2.0, accent),
                        );
                    }
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&tab_name)
                                .color(if is_active {
                                    Theme::TEXT_PRIMARY
                                } else {
                                    Theme::TEXT_MUTED
                                })
                                .size(12.0),
                        );

                        // 用 Label + Sense::click() 代替 Button，
                        // Label 不参与 Tab 键焦点循环，不会被意外激活。
                        let close_resp = ui.add(
                            egui::Label::new(RichText::new("×").color(Theme::TEXT_MUTED).size(13.0))
                                .sense(Sense::click()),
                        );
                        if close_resp.hovered() {
                            // 悬停时变亮，提示可点击
                            ui.painter().text(
                                close_resp.rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "×",
                                egui::FontId::proportional(13.0),
                                egui::Color32::from_rgb(243, 139, 168),
                            );
                        }
                        if close_resp.clicked() {
                            to_close = Some(tab_id.clone());
                        }
                    });
                });

            if tab_response
                .response
                .interact(Sense::click())
                .clicked()
            {
                app.active_tab = Some(tab_id.clone());
            }
        }

        if let Some(id) = to_close {
            app.open_tabs.retain(|t| t != &id);
            if app.active_tab.as_deref() == Some(id.as_str()) {
                app.active_tab = app.open_tabs.last().cloned();
            }
            // Kill PTY 进程并清理 running 状态
            if let Some(mut rs) = app.running.remove(&id) {
                if let Some(ref mut s) = rs.local {
                    s.kill();
                }
                // 同时清理分屏副 pane
                if let Some(split) = rs.split {
                    if let Some(mut sec) = app.running.remove(&split.session_id) {
                        if let Some(ref mut s) = sec.local {
                            s.kill();
                        }
                    }
                }
            }
        }

        // 「＋」同样用 Label，不获得键盘焦点
        let add_resp = ui.add(
            egui::Label::new(RichText::new("＋").color(Theme::TEXT_MUTED).size(14.0))
                .sense(Sense::click()),
        );
        if add_resp.hovered() {
            ui.painter().text(
                add_resp.rect.center(),
                egui::Align2::CENTER_CENTER,
                "＋",
                egui::FontId::proportional(14.0),
                Theme::ACCENT_LOCAL,
            );
        }
        // Plan 2: open new session dialog
    });
}

fn find_profile_name(groups: &[oxijade_config::SessionGroup], id: &str) -> Option<String> {
    for group in groups {
        for session in &group.sessions {
            if session.id() == id {
                return Some(session.name().to_string());
            }
        }
    }
    None
}

fn find_profile_accent(groups: &[oxijade_config::SessionGroup], id: &str) -> egui::Color32 {
    for group in groups {
        for session in &group.sessions {
            if session.id() == id {
                return match session {
                    SessionProfile::Local(_) => Theme::ACCENT_LOCAL,
                    SessionProfile::Ssh(_) => Theme::ACCENT_SSH,
                };
            }
        }
    }
    Theme::ACCENT_LOCAL
}
