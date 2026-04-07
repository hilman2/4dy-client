use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::config::HotkeyConfig;

pub fn register_hotkeys(app: &AppHandle, config: &HotkeyConfig) -> Result<(), String> {
    let gsm = app.global_shortcut();

    // Wählen aus Zwischenablage
    let app_handle = app.clone();
    gsm.on_shortcut(
        config.dial_clipboard.as_str(),
        move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                app_handle.emit("hotkey-action", "dial_clipboard").ok();
            }
        },
    )
    .map_err(|e| format!("Hotkey '{}': {}", config.dial_clipboard, e))?;

    // Wählen markierter Nummer
    let app_handle = app.clone();
    gsm.on_shortcut(
        config.dial_selection.as_str(),
        move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                app_handle.emit("hotkey-action", "dial_selection").ok();
            }
        },
    )
    .map_err(|e| format!("Hotkey '{}': {}", config.dial_selection, e))?;

    // Rufannahme
    let app_handle = app.clone();
    gsm.on_shortcut(
        config.answer_call.as_str(),
        move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                app_handle.emit("hotkey-action", "answer_call").ok();
            }
        },
    )
    .map_err(|e| format!("Hotkey '{}': {}", config.answer_call, e))?;

    // Auflegen
    let app_handle = app.clone();
    gsm.on_shortcut(config.hangup.as_str(), move |_app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            app_handle.emit("hotkey-action", "hangup").ok();
        }
    })
    .map_err(|e| format!("Hotkey '{}': {}", config.hangup, e))?;

    // Dialer öffnen (Fenster in Vordergrund + Dialer-Input fokussieren)
    let app_handle = app.clone();
    gsm.on_shortcut(
        config.open_dialer.as_str(),
        move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                app_handle.emit("hotkey-action", "open_dialer").ok();
            }
        },
    )
    .map_err(|e| format!("Hotkey '{}': {}", config.open_dialer, e))?;

    Ok(())
}
