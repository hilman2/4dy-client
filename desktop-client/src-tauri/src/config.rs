use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub pbx_url: String,

    #[serde(default)]
    pub mailto: MailtoConfig,

    #[serde(default)]
    pub hotkeys: HotkeyConfig,

    #[serde(default)]
    pub window: WindowConfig,

    #[serde(default)]
    pub user: Option<UserConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailtoConfig {
    #[serde(default = "default_subject_template")]
    pub subject_template: String,
    #[serde(default)]
    pub default_recipient: String,
}

impl Default for MailtoConfig {
    fn default() -> Self {
        Self {
            subject_template: default_subject_template(),
            default_recipient: String::new(),
        }
    }
}

fn default_subject_template() -> String {
    fourdy_i18n::t("mailto.subject").to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    #[serde(default = "default_dial_clipboard")]
    pub dial_clipboard: String,
    #[serde(default = "default_dial_selection")]
    pub dial_selection: String,
    #[serde(default = "default_answer_call")]
    pub answer_call: String,
    #[serde(default = "default_hangup")]
    pub hangup: String,
    #[serde(default = "default_open_dialer")]
    pub open_dialer: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            dial_clipboard: default_dial_clipboard(),
            dial_selection: default_dial_selection(),
            answer_call: default_answer_call(),
            hangup: default_hangup(),
            open_dialer: default_open_dialer(),
        }
    }
}

fn default_dial_clipboard() -> String {
    "Ctrl+F10".to_string()
}
fn default_dial_selection() -> String {
    "Ctrl+F11".to_string()
}
fn default_answer_call() -> String {
    "Ctrl+F9".to_string()
}
fn default_hangup() -> String {
    "Ctrl+F12".to_string()
}

fn default_open_dialer() -> String {
    "Ctrl+Num0".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default = "default_true")]
    pub remember_position: bool,
    #[serde(default)]
    pub start_minimized: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            remember_position: true,
            start_minimized: false,
        }
    }
}

fn default_width() -> u32 {
    800
}
fn default_height() -> u32 {
    600
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub name: String,
    pub extension: String,
}

/// Gibt den Config-Pfad zurück: %APPDATA%/4dy-client/config.json
pub fn config_path() -> PathBuf {
    let appdata = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    appdata.join("4dy-client").join("config.json")
}

/// Gibt den Pfad für die Fensterzustands-Datei zurück
pub fn window_state_path() -> PathBuf {
    let appdata = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    appdata.join("4dy-client").join("window-state.json")
}

/// Config laden
pub fn load_config() -> Result<AppConfig, String> {
    let path = config_path();
    if !path.exists() {
        return Err("Config-Datei nicht gefunden".to_string());
    }

    let content =
        fs::read_to_string(&path).map_err(|e| format!("Fehler beim Lesen der Config: {}", e))?;

    let config: AppConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Fehler beim Parsen der Config: {}", e))?;

    if config.pbx_url.is_empty() {
        return Err("pbx_url darf nicht leer sein".to_string());
    }

    Ok(config)
}

/// Erstellt eine minimale Config mit der PBX-URL
pub fn save_initial_config(pbx_url: &str) -> Result<AppConfig, String> {
    let config = AppConfig {
        pbx_url: pbx_url.to_string(),

        mailto: MailtoConfig::default(),
        hotkeys: HotkeyConfig::default(),
        window: WindowConfig::default(),
        user: None,
    };

    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Fehler beim Erstellen des Config-Verzeichnisses: {}", e))?;
    }

    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Fehler beim Serialisieren: {}", e))?;

    fs::write(&path, json).map_err(|e| format!("Fehler beim Schreiben der Config: {}", e))?;

    Ok(config)
}

/// Fensterzustand speichern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
}

pub fn save_window_state(state: &WindowState) -> Result<(), String> {
    let path = window_state_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let json =
        serde_json::to_string_pretty(state).map_err(|e| format!("Serialisierungsfehler: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Schreibfehler: {}", e))?;
    Ok(())
}

pub fn load_window_state() -> Option<WindowState> {
    let path = window_state_path();
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hotkey_defaults_match_documented_values() {
        let h = HotkeyConfig::default();
        assert_eq!(h.dial_clipboard, "Ctrl+F10");
        assert_eq!(h.dial_selection, "Ctrl+F11");
        assert_eq!(h.answer_call, "Ctrl+F9");
        assert_eq!(h.hangup, "Ctrl+F12");
        assert_eq!(h.open_dialer, "Ctrl+Num0");
    }

    #[test]
    fn window_defaults_are_800x600_remembered() {
        let w = WindowConfig::default();
        assert_eq!(w.width, 800);
        assert_eq!(w.height, 600);
        assert!(w.remember_position);
        assert!(!w.start_minimized);
    }

    #[test]
    fn minimal_config_fills_in_all_defaults() {
        let json = r#"{"pbx_url": "https://example.3cx.de"}"#;
        let cfg: AppConfig = serde_json::from_str(json).expect("must parse");

        assert_eq!(cfg.pbx_url, "https://example.3cx.de");
        assert_eq!(cfg.hotkeys.dial_clipboard, "Ctrl+F10");
        assert_eq!(cfg.window.width, 800);
        assert_eq!(cfg.window.height, 600);
        assert!(cfg.window.remember_position);
        assert!(!cfg.window.start_minimized);
        assert!(cfg.user.is_none());
    }

    #[test]
    fn full_config_roundtrip() {
        let json = r#"{
            "pbx_url": "https://acme.3cx.de",
            "hotkeys": {
                "dial_clipboard": "Ctrl+F1",
                "dial_selection": "Ctrl+F2",
                "answer_call": "Ctrl+F3",
                "hangup": "Ctrl+F4",
                "open_dialer": "Ctrl+F5"
            },
            "window": {
                "width": 1024,
                "height": 768,
                "remember_position": false,
                "start_minimized": true
            }
        }"#;
        let cfg: AppConfig = serde_json::from_str(json).expect("must parse");

        assert_eq!(cfg.hotkeys.dial_clipboard, "Ctrl+F1");
        assert_eq!(cfg.hotkeys.open_dialer, "Ctrl+F5");
        assert_eq!(cfg.window.width, 1024);
        assert_eq!(cfg.window.height, 768);
        assert!(!cfg.window.remember_position);
        assert!(cfg.window.start_minimized);
    }

    #[test]
    fn missing_pbx_url_fails_to_parse() {
        let json = r#"{}"#;
        let result: Result<AppConfig, _> = serde_json::from_str(json);
        assert!(result.is_err(), "pbx_url is required");
    }

    #[test]
    fn window_state_roundtrip() {
        let state = WindowState {
            x: 100,
            y: 200,
            width: 1280,
            height: 720,
            maximized: false,
        };
        let json = serde_json::to_string(&state).unwrap();
        let parsed: WindowState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.x, 100);
        assert_eq!(parsed.y, 200);
        assert_eq!(parsed.width, 1280);
        assert_eq!(parsed.height, 720);
        assert!(!parsed.maximized);
    }
}
