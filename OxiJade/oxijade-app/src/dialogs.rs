// oxijade-app/src/dialogs.rs
use oxijade_config::{SshAuth, SshProfile};

#[derive(Default)]
pub enum DialogState {
    #[default]
    None,
    SshForm(SshFormState),
}

#[derive(Clone)]
pub struct SshFormState {
    pub name: String,
    pub host: String,
    pub port: String,
    pub username: String,
    pub use_key: bool,
    pub key_path: String,
    pub password: String,
    pub proxy_jump: String,
    pub editing_id: Option<String>,
    pub error: Option<String>,
}

impl Default for SshFormState {
    fn default() -> Self {
        Self {
            name: String::new(),
            host: String::new(),
            port: "22".to_string(),
            username: String::new(),
            use_key: true,
            key_path: String::new(),
            password: String::new(),
            proxy_jump: String::new(),
            editing_id: None,
            error: None,
        }
    }
}

pub enum SshFormAction {
    None,
    Confirm(SshProfile),
    Cancel,
}

pub fn show_ssh_form(ctx: &egui::Context, state: &mut SshFormState) -> SshFormAction {
    let mut action = SshFormAction::None;

    egui::Window::new(if state.editing_id.is_some() {
        "编辑 SSH 会话"
    } else {
        "新建 SSH 会话"
    })
    .collapsible(false)
    .resizable(false)
    .min_width(420.0)
    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
    .show(ctx, |ui| {
        egui::Grid::new("ssh_form")
            .num_columns(2)
            .spacing([10.0, 6.0])
            .show(ui, |ui| {
                ui.label("名称");
                ui.text_edit_singleline(&mut state.name);
                ui.end_row();

                ui.label("主机地址");
                ui.text_edit_singleline(&mut state.host);
                ui.end_row();

                ui.label("端口");
                ui.text_edit_singleline(&mut state.port);
                ui.end_row();

                ui.label("用户名");
                ui.text_edit_singleline(&mut state.username);
                ui.end_row();

                ui.label("认证方式");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut state.use_key, true, "密钥");
                    ui.radio_value(&mut state.use_key, false, "密码");
                });
                ui.end_row();

                if state.use_key {
                    ui.label("密钥路径");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut state.key_path);
                        if ui.small_button("浏览…").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("选择私钥文件")
                                .pick_file()
                            {
                                state.key_path = path.to_string_lossy().to_string();
                            }
                        }
                    });
                    ui.end_row();
                } else {
                    ui.label("密码");
                    ui.add(egui::TextEdit::singleline(&mut state.password).password(true));
                    ui.end_row();
                }

                ui.label("跳板机（可选）");
                ui.text_edit_singleline(&mut state.proxy_jump);
                ui.end_row();
            });

        if let Some(err) = &state.error.clone() {
            ui.colored_label(egui::Color32::from_rgb(243, 139, 168), err);
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("确定").clicked() {
                if state.name.trim().is_empty() {
                    state.error = Some("请填写名称".into());
                } else if state.host.trim().is_empty() {
                    state.error = Some("请填写主机地址".into());
                } else if state.username.trim().is_empty() {
                    state.error = Some("请填写用户名".into());
                } else {
                    let port: u16 = state.port.trim().parse().unwrap_or(22);
                    let auth = if state.use_key {
                        SshAuth::Key {
                            path: state.key_path.trim().to_string(),
                        }
                    } else {
                        SshAuth::Password {
                            password: state.password.clone(),
                        }
                    };
                    let jump = state.proxy_jump.trim();
                    let profile = SshProfile {
                        id: state.editing_id.clone().unwrap_or_else(|| {
                            format!(
                                "ssh-{:x}",
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .map(|d| d.as_millis())
                                    .unwrap_or(0)
                            )
                        }),
                        name: state.name.trim().to_string(),
                        host: state.host.trim().to_string(),
                        port,
                        username: state.username.trim().to_string(),
                        auth,
                        proxy_jump: if jump.is_empty() {
                            None
                        } else {
                            Some(jump.to_string())
                        },
                    };
                    action = SshFormAction::Confirm(profile);
                }
            }
            if ui.button("取消").clicked() {
                action = SshFormAction::Cancel;
            }
        });
    });

    action
}

pub fn show_dialog(ctx: &egui::Context, app: &mut crate::app::OxiJadeApp) {
    let action = match &mut app.dialog {
        DialogState::SshForm(state) => show_ssh_form(ctx, state),
        DialogState::None => return,
    };
    match action {
        SshFormAction::None => {}
        SshFormAction::Cancel => {
            app.dialog = DialogState::None;
        }
        SshFormAction::Confirm(profile) => {
            use oxijade_config::{save_profiles, SessionProfile};
            // 重名检测：同名且不同 ID 的会话不允许保存
            let dup = app.profiles.groups.iter()
                .flat_map(|g| &g.sessions)
                .any(|s| s.name() == profile.name && s.id() != profile.id);
            if dup {
                if let DialogState::SshForm(ref mut state) = app.dialog {
                    state.error = Some(format!("名称「{}」已存在，请使用不同的名称", profile.name));
                }
                return;
            }
            let id = profile.id.clone();
            let ssh_profile = profile.clone();
            let mut found = false;
            for group in &mut app.profiles.groups {
                for sess in &mut group.sessions {
                    if sess.id() == id {
                        *sess = SessionProfile::Ssh(ssh_profile.clone());
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                let group = app.profiles.groups.iter_mut().find(|g| g.name == "SSH");
                if let Some(g) = group {
                    g.sessions.push(SessionProfile::Ssh(ssh_profile));
                } else {
                    app.profiles.groups.push(oxijade_config::SessionGroup {
                        name: "SSH".to_string(),
                        sessions: vec![SessionProfile::Ssh(ssh_profile)],
                    });
                }
            }
            let _ = save_profiles(&app.profiles);
            app.dialog = DialogState::None;
        }
    }
}
