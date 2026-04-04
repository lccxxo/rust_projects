#![allow(dead_code)]

use crate::dialogs::DialogState;
use crate::panels::sftp_panel::{SftpPanelState, SftpStatus};
use crate::theme::apply_theme;
use egui::{Context, RichText};
use oxijade_config::{load_profiles, LocalProfile, ProfileStore, SessionGroup, SessionProfile};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SplitDir {
    Horizontal, // 左右分屏
    Vertical,   // 上下分屏
}

#[derive(Clone, Debug)]
pub struct SplitState {
    pub session_id: String,    // 第二 pane 的会话 ID
    pub direction: SplitDir,
    pub ratio: f32,            // 主 pane 占比（0.15..0.85）
    pub focus_secondary: bool,
}

pub struct RunningSession {
    pub grid: Arc<Mutex<oxijade_core::terminal::TerminalGrid>>,
    pub local: Option<oxijade_core::session::local::LocalSession>,
    pub error: Option<String>,
    /// How many rows from the bottom the user has scrolled back (0 = live view).
    pub scroll_offset: usize,
    pub split: Option<SplitState>,
}

pub struct OxiJadeApp {
    pub active_tab: Option<String>,
    pub open_tabs: Vec<String>,
    pub sidebar_width: f32,
    pub profiles: ProfileStore,
    pub running: HashMap<String, RunningSession>,
    pub rt: tokio::runtime::Runtime,
    pub dialog: DialogState,
    pub sftp_panel: Option<SftpPanelState>,
}

impl Default for OxiJadeApp {
    fn default() -> Self {
        let mut profiles = load_profiles();
        if profiles.groups.is_empty() {
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
            running: HashMap::new(),
            rt: tokio::runtime::Runtime::new().unwrap(),
            dialog: DialogState::None,
            sftp_panel: None,
        }
    }
}

impl eframe::App for OxiJadeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        apply_theme(ctx);

        // 终端激活时处理 Tab 补全 + 防止 egui 焦点循环
        if self.active_tab.is_some() {
            // Fix 1: 先把 Tab 字节发给当前激活的终端 pane，再从事件队列移除
            let tab_count = ctx.input(|i| {
                i.events
                    .iter()
                    .filter(|e| {
                        matches!(
                            e,
                            egui::Event::Key {
                                key: egui::Key::Tab,
                                pressed: true,
                                ..
                            }
                        )
                    })
                    .count()
            });
            if tab_count > 0 {
                // 确定当前激活 pane 的会话 ID
                let session_id = if let Some(tab_id) = &self.active_tab {
                    if let Some(running) = self.running.get(tab_id) {
                        if let Some(split) = &running.split {
                            if split.focus_secondary {
                                split.session_id.clone()
                            } else {
                                tab_id.clone()
                            }
                        } else {
                            tab_id.clone()
                        }
                    } else {
                        tab_id.clone()
                    }
                } else {
                    String::new()
                };
                let writer = self
                    .running
                    .get(&session_id)
                    .and_then(|r| r.local.as_ref())
                    .map(|l| l.writer.clone());
                if let Some(w) = writer {
                    for _ in 0..tab_count {
                        let w2 = w.clone();
                        self.rt.spawn(async move {
                            let mut g = w2.lock().unwrap();
                            let _ = g.write_all(b"\t");
                            let _ = g.flush();
                        });
                    }
                }
            }
            // 从事件队列移除 Tab，防止 egui 焦点循环
            ctx.input_mut(|i| {
                i.events.retain(|e| {
                    !matches!(
                        e,
                        egui::Event::Key {
                            key: egui::Key::Tab,
                            pressed: true,
                            ..
                        }
                    )
                });
            });
            ctx.memory_mut(|m| m.stop_text_input());
        }

        egui::TopBottomPanel::top("tab_bar")
            .frame(
                egui::Frame::none()
                    .fill(crate::theme::Theme::BG_PANEL)
                    .inner_margin(egui::Margin::symmetric(0.0, 4.0)),
            )
            .show(ctx, |ui| {
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
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(
                            "Ctrl+Shift+F: SFTP  |  Ctrl+Shift+H/V: 分屏  |  Ctrl+Shift+W: 关闭分屏",
                        )
                        .color(crate::theme::Theme::TEXT_MUTED)
                        .size(10.0),
                    );
                });
            });
        });

        egui::SidePanel::left("sidebar")
            .default_width(self.sidebar_width)
            .width_range(150.0..=300.0)
            .show(ctx, |ui| {
                crate::panels::sidebar::show(ui, self);
            });

        // Ctrl+Shift+H 水平分屏，Ctrl+Shift+V 垂直分屏，Ctrl+Shift+W 关闭分屏，Ctrl+Shift+F SFTP 面板
        let kb = ctx.input(|i| {
            let cs = egui::Modifiers { ctrl: true, shift: true, ..Default::default() };
            let split_h = i.key_pressed(egui::Key::H) && i.modifiers == cs;
            let split_v = i.key_pressed(egui::Key::V) && i.modifiers == cs;
            let close_s = i.key_pressed(egui::Key::W) && i.modifiers == cs;
            let sftp_f = i.key_pressed(egui::Key::F) && i.modifiers == cs;
            (split_h, split_v, close_s, sftp_f)
        });

        // Ctrl+Shift+F 切换 SFTP 面板
        if kb.3 {
            if self.sftp_panel.is_some() {
                self.sftp_panel = None;
            } else {
                self.sftp_panel = Some(SftpPanelState {
                    session_id: self.active_tab.clone().unwrap_or_default(),
                    host: String::new(),
                    username: String::new(),
                    current_path: "/".into(),
                    entries: vec![],
                    status: SftpStatus::Disconnected,
                    password_input: String::new(),
                    show_password_dialog: false,
                    tx: None,
                    rx: None,
                });
            }
        }

        if let Some(tab_id) = self.active_tab.clone() {
            // 只允许对主 pane（非 -split 后缀）且尚未分屏的会话执行分屏
            let already_split = self
                .running
                .get(&tab_id)
                .map(|r| r.split.is_some())
                .unwrap_or(false);
            let is_secondary = tab_id.ends_with("-split");
            if (kb.0 || kb.1) && !kb.3 && !already_split && !is_secondary && self.running.contains_key(&tab_id) {
                let dir = if kb.0 { SplitDir::Horizontal } else { SplitDir::Vertical };
                let sec_id = format!("{tab_id}-split");
                if !self.running.contains_key(&sec_id) {
                    use oxijade_core::session::local::LocalSession;
                    use oxijade_core::session::SessionEvent;
                    use oxijade_core::terminal::TerminalGrid;

                    let grid = Arc::new(Mutex::new(TerminalGrid::new(80, 24)));
                    let grid_clone = grid.clone();
                    let (tx, mut rx) = tokio::sync::mpsc::channel::<SessionEvent>(256);
                    let _guard = self.rt.enter();
                    let result = LocalSession::new(
                        sec_id.clone(),
                        vec!["powershell.exe".to_string()],
                        tx,
                    );
                    let (session, error) = match result {
                        Ok(s) => (Some(s), None),
                        Err(e) => (None, Some(format!("{e:#}"))),
                    };
                    self.rt.spawn(async move {
                        while let Some(ev) = rx.recv().await {
                            match ev {
                                SessionEvent::Output(b) => {
                                    grid_clone.lock().unwrap().process_bytes(&b);
                                }
                                SessionEvent::Exited => break,
                            }
                        }
                    });
                    self.running.insert(
                        sec_id.clone(),
                        RunningSession {
                            grid,
                            local: session,
                            error,
                            scroll_offset: 0,
                            split: None,
                        },
                    );
                }
                if let Some(primary) = self.running.get_mut(&tab_id) {
                    primary.split = Some(SplitState {
                        session_id: sec_id,
                        direction: dir,
                        ratio: 0.5,
                        focus_secondary: false,
                    });
                }
            }
            if kb.2 {
                if let Some(primary) = self.running.get_mut(&tab_id) {
                    primary.split = None;
                }
            }
        }

        // SFTP 右侧面板
        if self.sftp_panel.is_some() {
            egui::SidePanel::right("sftp_panel")
                .min_width(280.0)
                .default_width(320.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("SFTP 文件传输")
                                .color(crate::theme::Theme::TEXT_PRIMARY)
                                .size(13.0)
                                .strong(),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Label::new(
                                        RichText::new("×")
                                            .size(14.0)
                                            .color(crate::theme::Theme::TEXT_MUTED),
                                    )
                                    .sense(egui::Sense::click()),
                                )
                                .clicked()
                            {
                                self.sftp_panel = None;
                            }
                        });
                    });
                    ui.separator();
                    crate::panels::sftp_panel::show(ui, self);
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            crate::panels::terminal::show(ui, self);
        });

        crate::dialogs::show_dialog(ctx, self);
    }
}
