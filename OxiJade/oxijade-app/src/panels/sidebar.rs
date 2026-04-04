// oxijade-app/src/panels/sidebar.rs
use crate::app::OxiJadeApp;
use crate::panels::sftp_panel::{SftpPanelState, SftpStatus};
use crate::theme::Theme;
use egui::{Color32, RichText, Ui};
use oxijade_config::{SessionProfile, SshAuth};
use std::io::Write;

enum SidebarAction {
    Delete(String),
    Edit(String),
}

pub fn show(ui: &mut Ui, app: &mut OxiJadeApp) {
    let mut pending: Option<SidebarAction> = None;

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
                        if let Some(action) = session_row(ui, session, app) {
                            pending = Some(action);
                        }
                    }
                },
            );
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        let add_resp = ui.add(
            egui::Label::new(
                RichText::new("＋ 新建 SSH 会话")
                    .color(Theme::ACCENT_SSH)
                    .size(12.0),
            )
            .sense(egui::Sense::click()),
        );
        if add_resp.clicked() {
            app.dialog = crate::dialogs::DialogState::SshForm(
                crate::dialogs::SshFormState::default(),
            );
        }
    });

    // 处理右键菜单动作（在 ScrollArea 外处理，避免重复借用）
    if let Some(action) = pending {
        match action {
            SidebarAction::Edit(profile_id) => {
                // 找到 profile 并填入 form
                let profile = app.profiles.groups.iter()
                    .flat_map(|g| &g.sessions)
                    .find(|s| s.id() == profile_id)
                    .cloned();
                if let Some(SessionProfile::Ssh(ssh)) = profile {
                    let (use_key, key_path, password) = match &ssh.auth {
                        SshAuth::Key { path } => (true, path.clone(), String::new()),
                        SshAuth::Password { password } => (false, String::new(), password.clone()),
                    };
                    app.dialog = crate::dialogs::DialogState::SshForm(
                        crate::dialogs::SshFormState {
                            name: ssh.name.clone(),
                            host: ssh.host.clone(),
                            port: ssh.port.to_string(),
                            username: ssh.username.clone(),
                            use_key,
                            key_path,
                            password,
                            proxy_jump: ssh.proxy_jump.clone().unwrap_or_default(),
                            editing_id: Some(ssh.id.clone()),
                            error: None,
                        },
                    );
                }
            }
            SidebarAction::Delete(profile_id) => {
                use oxijade_config::save_profiles;
                // 从 profiles 删除
                for group in &mut app.profiles.groups {
                    group.sessions.retain(|s| s.id() != profile_id);
                }
                // 删除空组（保留"本地"组）
                app.profiles.groups.retain(|g| !g.sessions.is_empty() || g.name == "本地");
                let _ = save_profiles(&app.profiles);

                // 关闭对应 tab
                app.open_tabs.retain(|t| t != &profile_id);
                if app.active_tab.as_deref() == Some(profile_id.as_str()) {
                    app.active_tab = app.open_tabs.last().cloned();
                }

                // Kill PTY
                if let Some(mut rs) = app.running.remove(&profile_id) {
                    if let Some(ref mut s) = rs.local {
                        s.kill();
                    }
                    if let Some(split) = rs.split {
                        if let Some(mut sec) = app.running.remove(&split.session_id) {
                            if let Some(ref mut s) = sec.local {
                                s.kill();
                            }
                        }
                    }
                }
            }
        }
    }
}

fn session_row(ui: &mut Ui, profile: &SessionProfile, app: &mut OxiJadeApp) -> Option<SidebarAction> {
    let id = profile.id().to_string();
    let name = profile.name().to_string();
    let (icon, accent) = match profile {
        SessionProfile::Local(_) => ("🖥", Theme::ACCENT_LOCAL),
        SessionProfile::Ssh(_) => ("🔗", Theme::ACCENT_SSH),
    };

    let is_active = app.active_tab.as_deref() == Some(id.as_str());
    let bg = if is_active { Theme::BG_SELECTED } else { Color32::TRANSPARENT };

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
                        .color(if is_active { Theme::TEXT_PRIMARY } else { Theme::TEXT_MUTED })
                        .size(12.0),
                );
                if is_active {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("●").color(accent).size(8.0));
                    });
                }
            });
        });

    let interact = frame_response.response.interact(egui::Sense::click());

    // 右键菜单
    let mut row_action: Option<SidebarAction> = None;
    interact.context_menu(|ui| {
        // 编辑（仅 SSH 会话）
        if matches!(profile, SessionProfile::Ssh(_)) {
            if ui.button("编辑").clicked() {
                row_action = Some(SidebarAction::Edit(id.clone()));
                ui.close_menu();
            }
        }
        if ui.button("删除会话").clicked() {
            row_action = Some(SidebarAction::Delete(id.clone()));
            ui.close_menu();
        }
    });

    if row_action.is_some() {
        return row_action;
    }

    // 单击打开/启动会话
    if interact.clicked() {
        if !app.open_tabs.contains(&id) {
            app.open_tabs.push(id.clone());
        }
        app.active_tab = Some(id.clone());

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

                let _guard = app.rt.enter();
                let session_result = LocalSession::new(
                    id.clone(),
                    vec![local_profile.shell.clone()],
                    tx,
                );

                let (session, error) = match session_result {
                    Ok(s) => {
                        // 切换 UTF-8 代码页（中文正常显示）
                        let w = s.writer.clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_millis(300));
                            let mut guard = w.lock().unwrap();
                            let _ = guard.write_all(b"chcp 65001\r\n");
                            let _ = guard.flush();
                        });
                        (Some(s), None)
                    }
                    Err(e) => (None, Some(format!("{e:#}"))),
                };

                app.rt.spawn(async move {
                    while let Some(event) = rx.recv().await {
                        match event {
                            SessionEvent::Output(bytes) => {
                                grid_clone.lock().unwrap().process_bytes(&bytes);
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
                        error,
                        scroll_offset: 0,
                        split: None,
                    },
                );
            } else if let SessionProfile::Ssh(ssh_profile) = profile {
                use crate::app::RunningSession;
                use oxijade_core::session::local::LocalSession;
                use oxijade_core::session::ssh::build_args;
                use oxijade_core::session::SessionEvent;
                use oxijade_core::terminal::TerminalGrid;
                use std::sync::{Arc, Mutex};

                let grid = Arc::new(Mutex::new(TerminalGrid::new(220, 50)));
                let grid_clone = grid.clone();
                let (tx, mut rx) = tokio::sync::mpsc::channel::<SessionEvent>(256);

                let _guard = app.rt.enter();
                let args = build_args(ssh_profile);
                let session_result = LocalSession::new(id.clone(), args, tx);

                let (session, error) = match session_result {
                    Ok(s) => (Some(s), None),
                    Err(e) => (None, Some(format!("{e:#}"))),
                };

                // 密码认证：在事件循环中监测 "password:" 提示后发送密码
                // 不再使用固定延迟，而是等到 ssh.exe 真正输出密码提示才发
                let password_to_send = match &ssh_profile.auth {
                    SshAuth::Password { password } if !password.is_empty() => {
                        Some(password.clone())
                    }
                    _ => None,
                };
                let writer_for_pwd = session.as_ref().map(|s| s.writer.clone());

                app.rt.spawn(async move {
                    let mut pwd_sent = false;
                    while let Some(event) = rx.recv().await {
                        match event {
                            SessionEvent::Output(bytes) => {
                                // 检测密码提示并自动发送
                                if !pwd_sent {
                                    if let (Some(ref pwd), Some(ref writer)) =
                                        (&password_to_send, &writer_for_pwd)
                                    {
                                        let text = String::from_utf8_lossy(&bytes);
                                        if text.to_lowercase().contains("assword:") {
                                            let mut g = writer.lock().unwrap();
                                            let _ = g.write_all(pwd.as_bytes());
                                            let _ = g.write_all(b"\r\n");
                                            let _ = g.flush();
                                            pwd_sent = true;
                                        }
                                    }
                                }
                                grid_clone.lock().unwrap().process_bytes(&bytes);
                            }
                            SessionEvent::Exited => break,
                        }
                    }
                });

                // 启动 SFTP 后台任务
                if session.is_some() {
                    use oxijade_core::sftp::{SftpRequest, SftpResponse};

                    let (sftp_req_tx, mut sftp_req_rx) =
                        tokio::sync::mpsc::channel::<SftpRequest>(32);
                    let (sftp_resp_tx, sftp_resp_rx) =
                        tokio::sync::mpsc::channel::<SftpResponse>(32);

                    let host = ssh_profile.host.clone();
                    let port = ssh_profile.port;
                    let username = ssh_profile.username.clone();
                    let auth = ssh_profile.auth.clone();

                    app.rt.spawn(async move {
                        let connect_result = match &auth {
                            SshAuth::Key { path } => {
                                oxijade_core::sftp::connect_key(&host, port, &username, path).await
                            }
                            SshAuth::Password { password } if !password.is_empty() => {
                                oxijade_core::sftp::connect_password(
                                    &host, port, &username, password,
                                )
                                .await
                            }
                            SshAuth::Password { .. } => {
                                sftp_resp_tx
                                    .send(SftpResponse::Error(
                                        "密码未保存：请编辑会话填写密码后重新点击连接".into(),
                                    ))
                                    .await
                                    .ok();
                                return;
                            }
                        };

                        match connect_result {
                            Err(e) => {
                                sftp_resp_tx
                                    .send(SftpResponse::Error(e.to_string()))
                                    .await
                                    .ok();
                            }
                            Ok(sftp_session) => {
                                let home =
                                    oxijade_core::sftp::home_dir(&sftp_session).await;
                                match oxijade_core::sftp::list_dir(&sftp_session, &home).await {
                                    Ok(entries) => {
                                        sftp_resp_tx
                                            .send(SftpResponse::DirListing {
                                                path: home,
                                                entries,
                                            })
                                            .await
                                            .ok();
                                    }
                                    Err(e) => {
                                        sftp_resp_tx
                                            .send(SftpResponse::Error(e.to_string()))
                                            .await
                                            .ok();
                                    }
                                }

                                while let Some(req) = sftp_req_rx.recv().await {
                                    match req {
                                        SftpRequest::ListDir(path) => {
                                            match oxijade_core::sftp::list_dir(
                                                &sftp_session,
                                                &path,
                                            )
                                            .await
                                            {
                                                Ok(entries) => {
                                                    sftp_resp_tx
                                                        .send(SftpResponse::DirListing {
                                                            path,
                                                            entries,
                                                        })
                                                        .await
                                                        .ok();
                                                }
                                                Err(e) => {
                                                    sftp_resp_tx
                                                        .send(SftpResponse::Error(e.to_string()))
                                                        .await
                                                        .ok();
                                                }
                                            }
                                        }
                                        SftpRequest::Download { remote, local } => {
                                            match oxijade_core::sftp::download(
                                                &sftp_session,
                                                &remote,
                                                &local,
                                            )
                                            .await
                                            {
                                                Ok(()) => {
                                                    sftp_resp_tx
                                                        .send(SftpResponse::DownloadDone { local })
                                                        .await
                                                        .ok();
                                                }
                                                Err(e) => {
                                                    sftp_resp_tx
                                                        .send(SftpResponse::Error(e.to_string()))
                                                        .await
                                                        .ok();
                                                }
                                            }
                                        }
                                        SftpRequest::Upload { local, remote } => {
                                            match oxijade_core::sftp::upload(
                                                &sftp_session,
                                                &local,
                                                &remote,
                                            )
                                            .await
                                            {
                                                Ok(()) => {
                                                    sftp_resp_tx
                                                        .send(SftpResponse::UploadDone)
                                                        .await
                                                        .ok();
                                                }
                                                Err(e) => {
                                                    sftp_resp_tx
                                                        .send(SftpResponse::Error(e.to_string()))
                                                        .await
                                                        .ok();
                                                }
                                            }
                                        }
                                        SftpRequest::Disconnect => break,
                                    }
                                }
                            }
                        }
                    });

                    app.sftp_panel = Some(SftpPanelState {
                        session_id: id.clone(),
                        host: ssh_profile.host.clone(),
                        username: ssh_profile.username.clone(),
                        current_path: "~".into(),
                        entries: vec![],
                        status: SftpStatus::Connecting,
                        password_input: String::new(),
                        show_password_dialog: false,
                        tx: Some(sftp_req_tx),
                        rx: Some(sftp_resp_rx),
                    });
                }

                app.running.insert(
                    id,
                    RunningSession {
                        grid,
                        local: session,
                        error,
                        scroll_offset: 0,
                        split: None,
                    },
                );
            }
        }
    }

    None
}
