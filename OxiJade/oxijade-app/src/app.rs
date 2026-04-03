#![allow(dead_code)]

use crate::dialogs::DialogState;
use crate::theme::apply_theme;
use egui::Context;
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
        }
    }
}

impl eframe::App for OxiJadeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        apply_theme(ctx);

        // 终端激活时：
        // 1. 从事件队列删除 Tab 按下事件，防止 egui 焦点循环
        // 2. 清除当前已有的键盘焦点，确保没有 UI 控件处于聚焦状态
        if self.active_tab.is_some() {
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
            // 强制清除 UI 焦点，即使 Tab 被其他路径处理也不会停留在按钮上
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
            });
        });

        egui::SidePanel::left("sidebar")
            .default_width(self.sidebar_width)
            .width_range(150.0..=300.0)
            .show(ctx, |ui| {
                crate::panels::sidebar::show(ui, self);
            });

        // Ctrl+Shift+H 水平分屏，Ctrl+Shift+V 垂直分屏，Ctrl+Shift+W 关闭分屏
        let kb = ctx.input(|i| {
            let cs = egui::Modifiers { ctrl: true, shift: true, ..Default::default() };
            let split_h = i.key_pressed(egui::Key::H) && i.modifiers == cs;
            let split_v = i.key_pressed(egui::Key::V) && i.modifiers == cs;
            let close_s = i.key_pressed(egui::Key::W) && i.modifiers == cs;
            (split_h, split_v, close_s)
        });

        if let Some(tab_id) = self.active_tab.clone() {
            if (kb.0 || kb.1) && self.running.contains_key(&tab_id) {
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

        egui::CentralPanel::default().show(ctx, |ui| {
            crate::panels::terminal::show(ui, self);
        });

        crate::dialogs::show_dialog(ctx, self);
    }
}
