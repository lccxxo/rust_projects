// oxijade-app/src/panels/sidebar.rs
use crate::app::OxiJadeApp;
use crate::theme::Theme;
use egui::{Color32, RichText, Ui};
use oxijade_config::SessionProfile;
use std::io::Write;

pub fn show(ui: &mut Ui, app: &mut OxiJadeApp) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(6.0);

        let profiles = app.profiles.clone();

        for group in &profiles.groups {
            ui.collapsing(
                RichText::new(format!("📁 {}", group.name))
                    .color(Theme::TEXT_MUTED)
                    .size(12.0),
                |ui| {
                    for session in &group.sessions {
                        session_row(ui, session, app);
                    }
                },
            );
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        if ui
            .button(
                RichText::new("＋ 新建会话")
                    .color(Theme::ACCENT_LOCAL)
                    .size(12.0),
            )
            .clicked()
        {
            // Plan 2: open new session dialog
        }
    });
}

fn session_row(ui: &mut Ui, profile: &SessionProfile, app: &mut OxiJadeApp) {
    let id = profile.id().to_string();
    let name = profile.name().to_string();
    let (icon, accent) = match profile {
        SessionProfile::Local(_) => ("🖥", Theme::ACCENT_LOCAL),
        SessionProfile::Ssh(_) => ("🔗", Theme::ACCENT_SSH),
    };

    let is_active = app.active_tab.as_deref() == Some(id.as_str());
    let bg = if is_active {
        Theme::BG_SELECTED
    } else {
        Color32::TRANSPARENT
    };

    let frame_response = egui::Frame::none()
        .fill(bg)
        .rounding(4.0)
        .inner_margin(egui::Margin::symmetric(8.0, 3.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon).size(12.0));
                ui.label(
                    RichText::new(&name)
                        .color(if is_active {
                            Theme::TEXT_PRIMARY
                        } else {
                            Theme::TEXT_MUTED
                        })
                        .size(12.0),
                );
                if is_active {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("●").color(accent).size(8.0));
                    });
                }
            });
        });

    if frame_response
        .response
        .interact(egui::Sense::click())
        .clicked()
    {
        if !app.open_tabs.contains(&id) {
            app.open_tabs.push(id.clone());
        }
        app.active_tab = Some(id.clone());

        // Launch PTY session if not already running
        if !app.running.contains_key(&id) {
            if let SessionProfile::Local(local_profile) = profile {
                use crate::app::RunningSession;
                use oxijade_core::session::local::LocalSession;
                use oxijade_core::session::SessionEvent;
                use oxijade_core::terminal::TerminalGrid;
                use std::sync::{Arc, Mutex};

                let grid = Arc::new(Mutex::new(TerminalGrid::new(220, 50)));
                let grid_clone = grid.clone();
                let (tx, mut rx) = tokio::sync::mpsc::channel::<SessionEvent>(256);
                let session = LocalSession::new(id.clone(), local_profile.shell.clone(), tx).ok();

                // Switch terminal to UTF-8 so Chinese output renders correctly
                if let Some(ref s) = session {
                    let w = s.writer.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(300));
                        let mut guard = w.lock().unwrap();
                        let _ = guard.write_all(b"chcp 65001\r\n");
                        let _ = guard.flush();
                    });
                }

                app.rt.spawn(async move {
                    while let Some(event) = rx.recv().await {
                        match event {
                            SessionEvent::Output(bytes) => {
                                let mut g = grid_clone.lock().unwrap();
                                g.process_bytes(&bytes);
                            }
                            SessionEvent::Exited => break,
                        }
                    }
                });

                app.running.insert(
                    id,
                    RunningSession {
                        grid,
                        local: session,
                    },
                );
            }
        }
    }
}
