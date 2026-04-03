// oxijade-app/src/panels/sftp_panel.rs
use crate::app::OxiJadeApp;
use crate::theme::Theme;
use egui::{RichText, Ui};
use oxijade_core::sftp::{SftpEntry, SftpRequest, SftpResponse};

pub enum SftpStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

pub struct SftpPanelState {
    pub session_id: String,
    pub host: String,
    pub username: String,
    pub current_path: String,
    pub entries: Vec<SftpEntry>,
    pub status: SftpStatus,
    pub password_input: String,
    pub show_password_dialog: bool,
    pub tx: Option<tokio::sync::mpsc::Sender<SftpRequest>>,
    pub rx: Option<tokio::sync::mpsc::Receiver<SftpResponse>>,
}

pub fn show(ui: &mut Ui, app: &mut OxiJadeApp) {
    let Some(state) = app.sftp_panel.as_mut() else {
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("无 SFTP 连接").color(Theme::TEXT_MUTED));
        });
        return;
    };

    // 轮询后台响应
    if let Some(rx) = state.rx.as_mut() {
        while let Ok(resp) = rx.try_recv() {
            match resp {
                SftpResponse::DirListing { path, entries } => {
                    state.current_path = path;
                    state.entries = entries;
                    state.status = SftpStatus::Connected;
                }
                SftpResponse::DownloadDone { local: _ } => {
                    state.status = SftpStatus::Connected;
                }
                SftpResponse::UploadDone => {
                    if let Some(tx) = state.tx.as_ref() {
                        let _ = tx.blocking_send(SftpRequest::ListDir(state.current_path.clone()));
                    }
                }
                SftpResponse::Error(e) => {
                    state.status = SftpStatus::Error(e);
                }
            }
        }
    }

    // 顶栏：路径 + 按钮
    ui.horizontal(|ui| {
        ui.label(RichText::new("📂").size(14.0));
        ui.label(
            RichText::new(&state.current_path)
                .color(Theme::TEXT_PRIMARY)
                .size(12.0),
        );
    });
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("上传").clicked() {
                if let Some(path) = rfd::FileDialog::new().set_title("选择上传文件").pick_file() {
                    let remote = format!(
                        "{}/{}",
                        state.current_path.trim_end_matches('/'),
                        path.file_name().unwrap_or_default().to_string_lossy()
                    );
                    if let Some(tx) = state.tx.as_ref() {
                        let _ = tx.blocking_send(SftpRequest::Upload { local: path, remote });
                    }
                }
            }
            if ui.small_button("刷新").clicked() {
                if let Some(tx) = state.tx.as_ref() {
                    let _ = tx.blocking_send(SftpRequest::ListDir(state.current_path.clone()));
                }
            }
            if state.current_path != "/" && ui.small_button("↑").clicked() {
                let parent = std::path::Path::new(&state.current_path)
                    .parent()
                    .map(|p| p.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_else(|| "/".to_string());
                let parent = if parent.is_empty() {
                    "/".to_string()
                } else {
                    parent
                };
                if let Some(tx) = state.tx.as_ref() {
                    let _ = tx.blocking_send(SftpRequest::ListDir(parent));
                }
            }
        });
    });

    ui.separator();

    // 状态行
    match &state.status {
        SftpStatus::Connecting => {
            ui.label(RichText::new("连接中…").color(Theme::TEXT_MUTED).size(11.0));
        }
        SftpStatus::Error(e) => {
            ui.label(
                RichText::new(format!("✗ {e}"))
                    .color(egui::Color32::from_rgb(243, 139, 168))
                    .size(11.0),
            );
        }
        _ => {}
    }

    // 文件列表
    egui::ScrollArea::vertical().show(ui, |ui| {
        let entries = state.entries.clone();
        for entry in &entries {
            let icon = if entry.is_dir { "📁" } else { "📄" };
            let label_text = format!("{icon} {}", entry.name);
            let row_resp = ui.add(
                egui::Label::new(
                    RichText::new(&label_text)
                        .color(if entry.is_dir {
                            Theme::ACCENT_LOCAL
                        } else {
                            Theme::TEXT_PRIMARY
                        })
                        .size(12.0),
                )
                .sense(egui::Sense::click()),
            );

            if row_resp.double_clicked() {
                if entry.is_dir {
                    if let Some(tx) = state.tx.as_ref() {
                        let _ = tx.blocking_send(SftpRequest::ListDir(entry.full_path.clone()));
                    }
                } else {
                    let local = dirs_next::download_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                        .join(&entry.name);
                    if let Some(tx) = state.tx.as_ref() {
                        let _ = tx.blocking_send(SftpRequest::Download {
                            remote: entry.full_path.clone(),
                            local,
                        });
                    }
                }
            }

            if !entry.is_dir {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format_size(entry.size))
                            .color(Theme::TEXT_MUTED)
                            .size(10.0),
                    );
                });
            }
        }
    });

    // 拖放上传
    let dropped: Vec<_> = ui.ctx().input(|i| i.raw.dropped_files.clone());
    if !dropped.is_empty() {
        for file in dropped {
            if let Some(path) = file.path {
                let remote = format!(
                    "{}/{}",
                    state.current_path.trim_end_matches('/'),
                    path.file_name().unwrap_or_default().to_string_lossy()
                );
                if let Some(tx) = state.tx.as_ref() {
                    let _ = tx.blocking_send(SftpRequest::Upload { local: path, remote });
                }
            }
        }
    }
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
