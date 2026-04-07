use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

/// Tray-Tooltip mit Badge-Zähler aktualisieren
pub fn update_badge(app: &AppHandle, count: u32) {
    if let Some(tray) = app.tray_by_id("main") {
        let tooltip = if count > 0 {
            format!("4dy Client ({} offen)", count)
        } else {
            "4dy Client".to_string()
        };
        tray.set_tooltip(Some(&tooltip)).ok();
    }
}

fn show_window(app: &AppHandle) {
    let window = app
        .get_webview_window("webclient")
        .or_else(|| app.get_webview_window("main"));
    if let Some(w) = window {
        w.show().ok();
        w.unminimize().ok();
        w.set_focus().ok();
    }
}

pub fn setup_tray(app: &AppHandle) -> Result<(), String> {
    let show = MenuItem::with_id(app, "show", fourdy_i18n::t("tray.show"), true, None::<&str>)
        .map_err(|e| format!("Menü-Fehler: {}", e))?;
    let reload_config = MenuItem::with_id(
        app,
        "reload_config",
        fourdy_i18n::t("tray.reload_config"),
        true,
        None::<&str>,
    )
    .map_err(|e| format!("Menü-Fehler: {}", e))?;
    let open_config = MenuItem::with_id(
        app,
        "open_config",
        fourdy_i18n::t("tray.open_config"),
        true,
        None::<&str>,
    )
    .map_err(|e| format!("Menü-Fehler: {}", e))?;
    let register_tel = MenuItem::with_id(
        app,
        "register_tel",
        fourdy_i18n::t("tray.register_tel"),
        true,
        None::<&str>,
    )
    .map_err(|e| format!("Menü-Fehler: {}", e))?;
    let devtools = MenuItem::with_id(
        app,
        "devtools",
        fourdy_i18n::t("tray.devtools"),
        true,
        None::<&str>,
    )
    .map_err(|e| format!("Menü-Fehler: {}", e))?;
    let separator = MenuItem::with_id(app, "sep", "─────────", false, None::<&str>)
        .map_err(|e| format!("Menü-Fehler: {}", e))?;
    let quit = MenuItem::with_id(app, "quit", fourdy_i18n::t("tray.quit"), true, None::<&str>)
        .map_err(|e| format!("Menü-Fehler: {}", e))?;

    let menu = Menu::with_items(
        app,
        &[
            &show,
            &reload_config,
            &open_config,
            &register_tel,
            &devtools,
            &separator,
            &quit,
        ],
    )
    .map_err(|e| format!("Menü-Fehler: {}", e))?;

    let _tray = TrayIconBuilder::with_id("main")
        .icon(tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png")).unwrap())
        .menu(&menu)
        .tooltip("4dy Client")
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "show" => {
                show_window(app);
            }
            "reload_config" => match crate::config::load_config() {
                Ok(_config) => {
                    tauri::Emitter::emit(app, "config-reloaded", ()).ok();
                }
                Err(e) => {
                    eprintln!("Config-Reload fehlgeschlagen: {}", e);
                }
            },
            "open_config" => {
                let config_path = crate::config::config_path();
                #[cfg(windows)]
                {
                    std::process::Command::new("cmd")
                        .args(["/C", "start", "", &config_path.to_string_lossy()])
                        .spawn()
                        .ok();
                }
            }
            "register_tel" => match crate::register_tel_handler() {
                Ok(_) => println!("[4dy] tel: Handler erfolgreich registriert"),
                Err(e) => eprintln!("[4dy] tel: Handler Fehler: {}", e),
            },
            "devtools" =>
            {
                #[cfg(debug_assertions)]
                if let Some(webview) = app.get_webview_window("webclient") {
                    if webview.is_devtools_open() {
                        webview.close_devtools();
                    } else {
                        webview.open_devtools();
                    }
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_window(tray.app_handle());
            }
        })
        .build(app)
        .map_err(|e| format!("Tray-Fehler: {}", e))?;

    Ok(())
}
