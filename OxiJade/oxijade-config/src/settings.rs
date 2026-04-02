use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybindings {
    pub new_tab: String,
    pub close_tab: String,
    pub split_horizontal: String,
    pub split_vertical: String,
    pub search: String,
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            new_tab: "Ctrl+T".to_string(),
            close_tab: "Ctrl+W".to_string(),
            split_horizontal: "Ctrl+Shift+H".to_string(),
            split_vertical: "Ctrl+Shift+V".to_string(),
            search: "Ctrl+F".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub font_family: String,
    pub font_size: f32,
    pub keybindings: Keybindings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            font_family: "JetBrains Mono".to_string(),
            font_size: 14.0,
            keybindings: Keybindings::default(),
        }
    }
}

/// Returns the config directory: %APPDATA%\OxiJade\
pub fn config_dir() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("OxiJade")
}

pub fn load_settings() -> AppSettings {
    let path = config_dir().join("settings.json");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_settings(settings: &AppSettings) -> std::io::Result<()> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    let json = serde_json::to_string_pretty(settings).unwrap();
    std::fs::write(dir.join("settings.json"), json)
}

pub fn load_profiles() -> crate::profile::ProfileStore {
    let path = config_dir().join("profiles.json");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_profiles(store: &crate::profile::ProfileStore) -> std::io::Result<()> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    let json = serde_json::to_string_pretty(store).unwrap();
    std::fs::write(dir.join("profiles.json"), json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_defaults() {
        let s = AppSettings::default();
        assert_eq!(s.font_size, 14.0);
        assert_eq!(s.font_family, "JetBrains Mono");
    }

    #[test]
    fn test_settings_json_roundtrip() {
        let s = AppSettings::default();
        let json = serde_json::to_string(&s).unwrap();
        let decoded: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.font_size, s.font_size);
        assert_eq!(decoded.keybindings.new_tab, s.keybindings.new_tab);
    }
}
