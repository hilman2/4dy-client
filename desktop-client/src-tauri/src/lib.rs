pub mod clipboard;
pub mod config;
pub mod hotkeys;
pub mod inject;
pub mod tray;

use config::{AppConfig, WindowState};
use tauri::{Emitter, Listener, Manager, WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
fn get_config() -> Result<AppConfig, String> {
    config::load_config()
}

#[tauri::command]
fn save_initial_config(pbx_url: String, app: tauri::AppHandle) -> Result<AppConfig, String> {
    let config = config::save_initial_config(&pbx_url)?;

    if let Err(e) = hotkeys::register_hotkeys(&app, &config.hotkeys) {
        eprintln!("Hotkey-Registrierung fehlgeschlagen: {}", e);
    }

    // Config geschrieben → App beenden, User startet neu
    #[cfg(windows)]
    {
        use windows::Win32::UI::WindowsAndMessaging::*;
        let msg = to_wide(fourdy_i18n::t("setup.restart_msg"));
        let title = to_wide(fourdy_i18n::t("setup.restart_title"));
        unsafe {
            MessageBoxW(
                Some(windows::Win32::Foundation::HWND::default()),
                windows::core::PCWSTR(msg.as_ptr()),
                windows::core::PCWSTR(title.as_ptr()),
                MB_OK | MB_ICONINFORMATION,
            );
        }
    }
    app.exit(0);
    Ok(config)
}

#[tauri::command]
fn clean_phone_number(input: String) -> Option<String> {
    clipboard::extract_phone_number(&input)
}

#[tauri::command]
fn save_window_state(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    maximized: bool,
) -> Result<(), String> {
    let state = WindowState {
        x,
        y,
        width,
        height,
        maximized,
    };
    config::save_window_state(&state)
}

/// Wird von der JS-Injection / CRM-Integration aufgerufen wenn ein Anruf eingeht
#[tauri::command]
fn on_incoming_call(caller_name: String, caller_number: String, app: tauri::AppHandle) {
    println!(
        "[4dy] Eingehender Anruf: {} ({})",
        caller_name, caller_number
    );
    show_call_notification(&app, &caller_name, &caller_number);
    app.emit(
        "incoming-call",
        serde_json::json!({
            "name": caller_name,
            "number": caller_number,
        }),
    )
    .ok();
}

/// DevTools öffnen/schließen
#[tauri::command]
fn toggle_devtools(app: tauri::AppHandle) {
    #[cfg(debug_assertions)]
    if let Some(webview) = app.get_webview_window("webclient") {
        if webview.is_devtools_open() {
            webview.close_devtools();
        } else {
            webview.open_devtools();
        }
    }
}

/// Bei eingehendem Anruf: Fenster als kompakten Dialer unten rechts zeigen
#[tauri::command]
fn show_notification(title: String, body: String, app: tauri::AppHandle) {
    println!("[4dy] show_notification aufgerufen: '{}' '{}'", title, body);
    if let Some(window) = app.get_webview_window("webclient") {
        let is_focused = window.is_focused().unwrap_or(false);
        let is_visible = window.is_visible().unwrap_or(false);
        let is_minimized = window.is_minimized().unwrap_or(false);

        println!(
            "[4dy] Fenster-Status: focused={}, visible={}, minimized={}",
            is_focused, is_visible, is_minimized
        );

        if is_focused {
            println!("[4dy] Fenster hat Fokus → übersprungen");
            return;
        }

        let needs_compact = !is_visible || is_minimized;
        println!(
            "[4dy] Bringe Fenster in den Vordergrund (kompakt={})",
            needs_compact
        );

        window.show().ok();
        window.unminimize().ok();

        if needs_compact {
            // Aus Tray/Minimiert → kompakt unten rechts positionieren + Dialer öffnen
            #[cfg(windows)]
            {
                use windows::Win32::Foundation::RECT;
                use windows::Win32::UI::WindowsAndMessaging::*;
                unsafe {
                    let mut rc = RECT::default();
                    SystemParametersInfoW(
                        SPI_GETWORKAREA,
                        0,
                        Some(&mut rc as *mut _ as *mut _),
                        SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
                    )
                    .ok();
                    let sx = rc.right;
                    let sy = rc.bottom;
                    let win_w = 760u32;
                    let win_h = std::cmp::min(820, (sy - rc.top - 10) as u32);
                    let x = sx - win_w as i32 - 8;
                    let y = sy - win_h as i32 - 200;

                    window
                        .set_size(tauri::Size::Physical(tauri::PhysicalSize::new(
                            win_w, win_h,
                        )))
                        .ok();
                    window
                        .set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(
                            x, y,
                        )))
                        .ok();
                }
            }
            // Dialer öffnen falls geschlossen
            window
                .eval(
                    "if (!document.querySelector('.openContainer')) { \
                     var btn = document.getElementById('menuDialer'); \
                     if (btn) btn.click(); \
                 }",
                )
                .ok();
        }

        // Fenster nach vorne holen ohne Fokus zu klauen
        #[cfg(windows)]
        {
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::WindowsAndMessaging::*;
            if let Ok(raw) = window.hwnd() {
                unsafe {
                    let hwnd = HWND(raw.0);
                    SetWindowPos(
                        hwnd,
                        Some(HWND_TOPMOST),
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                    )
                    .ok();
                    SetWindowPos(
                        hwnd,
                        Some(HWND_NOTOPMOST),
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                    )
                    .ok();
                }
            }
        }
    }
}

/// Registriert die App als tel: Protocol Handler in der Windows Registry
#[tauri::command]
fn register_tel_handler() -> Result<(), String> {
    #[cfg(windows)]
    {
        // tel-handler.exe neben der eigenen exe suchen, Fallback: Dev-Pfad
        let own_exe =
            std::env::current_exe().map_err(|e| format!("Exe-Pfad nicht gefunden: {}", e))?;
        let own_dir = own_exe.parent().unwrap();

        let handler_exe = if own_dir.join("tel-handler.exe").exists() {
            own_dir.join("tel-handler.exe")
        } else {
            let dev_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("tools/tel-handler/target/debug/tel-handler.exe");
            if dev_path.exists() {
                dev_path
            } else {
                return Err("tel-handler.exe nicht gefunden! Bitte bauen: cd tools/tel-handler && cargo build".into());
            }
        };

        let exe_path = handler_exe.to_string_lossy().to_string();
        let command = format!("\"{}\" \"%1\"", exe_path);

        use std::process::Command;

        // 1) ProgID registrieren: HKCU\Software\Classes\4dyClient.tel
        Command::new("reg")
            .args([
                "add",
                "HKCU\\Software\\Classes\\4dyClient.tel",
                "/ve",
                "/d",
                "URL:Tel Protocol",
                "/f",
            ])
            .output()
            .ok();
        Command::new("reg")
            .args([
                "add",
                "HKCU\\Software\\Classes\\4dyClient.tel",
                "/v",
                "URL Protocol",
                "/d",
                "",
                "/f",
            ])
            .output()
            .ok();
        Command::new("reg")
            .args([
                "add",
                "HKCU\\Software\\Classes\\4dyClient.tel\\shell\\open\\command",
                "/ve",
                "/d",
                &command,
                "/f",
            ])
            .output()
            .ok();

        // 2) Capabilities registrieren
        Command::new("reg")
            .args([
                "add",
                "HKCU\\Software\\4dyClient\\Capabilities",
                "/v",
                "ApplicationName",
                "/d",
                "4dy Client",
                "/f",
            ])
            .output()
            .ok();
        Command::new("reg")
            .args([
                "add",
                "HKCU\\Software\\4dyClient\\Capabilities",
                "/v",
                "ApplicationDescription",
                "/d",
                "4dy Custom Desktop Client",
                "/f",
            ])
            .output()
            .ok();
        Command::new("reg")
            .args([
                "add",
                "HKCU\\Software\\4dyClient\\Capabilities\\URLAssociations",
                "/v",
                "tel",
                "/d",
                "4dyClient.tel",
                "/f",
            ])
            .output()
            .ok();

        // 3) App als RegisteredApplication eintragen
        Command::new("reg")
            .args([
                "add",
                "HKCU\\Software\\RegisteredApplications",
                "/v",
                "4dyClient",
                "/d",
                "Software\\4dyClient\\Capabilities",
                "/f",
            ])
            .output()
            .ok();

        // 4) Auch direkt unter tel: setzen (Fallback für ältere Windows-Versionen)
        Command::new("reg")
            .args([
                "add",
                "HKCU\\Software\\Classes\\tel",
                "/ve",
                "/d",
                "URL:Tel Protocol",
                "/f",
            ])
            .output()
            .ok();
        Command::new("reg")
            .args([
                "add",
                "HKCU\\Software\\Classes\\tel",
                "/v",
                "URL Protocol",
                "/d",
                "",
                "/f",
            ])
            .output()
            .ok();
        Command::new("reg")
            .args([
                "add",
                "HKCU\\Software\\Classes\\tel\\shell\\open\\command",
                "/ve",
                "/d",
                &command,
                "/f",
            ])
            .output()
            .ok();

        // 5) Windows Standard-Apps Einstellungen öffnen damit User die App auswählen kann
        Command::new("cmd")
            .args(["/C", "start", "ms-settings:defaultapps"])
            .spawn()
            .ok();

        println!("[4dy] tel: Handler registriert: {}", exe_path);
        println!("[4dy] Bitte in den Windows-Einstellungen '4dy Client' als Standard für tel: Links auswählen");
        Ok(())
    }
    #[cfg(not(windows))]
    Err("Nur auf Windows unterstützt".to_string())
}

/// Wählt eine Nummer im Web-Client
#[tauri::command]
fn dial_number(number: String, app: tauri::AppHandle) {
    if let Some(webview) = app.get_webview_window("webclient") {
        let cleaned = clipboard::extract_phone_number(&number).unwrap_or(number);
        let js = format!("window.__4DY_DIAL && window.__4DY_DIAL('{}');", cleaned);
        webview.eval(&js).ok();
    }
}

// ── Fenster-Helfer ───────────────────────────────────────────

/// Startet die standalone callback-popup.exe mit Anrufer-Info
fn show_call_notification(_app: &tauri::AppHandle, caller_name: &str, caller_number: &str) {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));

    // 1) Neben der eigenen exe suchen (Prod)
    let mut exe = exe_dir
        .as_ref()
        .map(|d| d.join("callback-popup.exe"))
        .unwrap_or_else(|| std::path::PathBuf::from("callback-popup.exe"));

    // 2) Dev-Fallback: im tools-Verzeichnis
    if !exe.exists() {
        let dev_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("tools/callback-popup/target/debug/callback-popup.exe");
        if dev_path.exists() {
            exe = dev_path;
        } else {
            eprintln!("[4dy] callback-popup.exe nicht gefunden!");
            eprintln!("[4dy] Bitte erst bauen: cd tools/callback-popup && cargo build");
            return;
        }
    }

    println!(
        "[4dy] Starte Rückruf-Popup: {} ({})",
        caller_name, caller_number
    );

    match std::process::Command::new(&exe)
        .arg(caller_name)
        .arg(caller_number)
        .spawn()
    {
        Ok(_) => {}
        Err(e) => eprintln!("[4dy] callback-popup.exe Fehler: {}", e),
    }
}

/// Erstellt das Web-Client-Fenster mit der konfigurierten URL
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn create_webclient_window(app: &tauri::AppHandle, config: &AppConfig) -> Result<(), String> {
    let url: url::Url = config
        .pbx_url
        .parse()
        .map_err(|e| format!("Ungültige PBX-URL: {}", e))?;

    let mut builder = WebviewWindowBuilder::new(app, "webclient", WebviewUrl::External(url))
        .title("4dy Client")
        .inner_size(config.window.width as f64, config.window.height as f64)
        .min_inner_size(800.0, 600.0)
        .resizable(true)
        .center()
        .devtools(true)
        .initialization_script(inject::get_csp_override())
        .initialization_script(inject::get_notification_polyfill())
        .initialization_script(inject::get_injection_script());

    if config.window.remember_position {
        if let Some(state) = config::load_window_state() {
            builder = builder
                .position(state.x as f64, state.y as f64)
                .inner_size(state.width as f64, state.height as f64);
        }
    }

    let window = builder
        .build()
        .map_err(|e| format!("Fenster konnte nicht erstellt werden: {}", e))?;

    // WebView2: Permissions erlauben + CRM-URLs abfangen
    #[cfg(windows)]
    {
        let app_handle = app.clone();
        window
            .with_webview(move |webview| {
                unsafe {
                    use webview2_com::Microsoft::Web::WebView2::Win32::*;
                    let core = webview.controller().CoreWebView2().unwrap();
                    let mut token: i64 = 0;

                    // Alle Permission-Requests automatisch erlauben
                    core.add_PermissionRequested(
                        &webview2_com::PermissionRequestedEventHandler::create(Box::new(
                            |_, args| {
                                if let Some(args) = args {
                                    args.SetState(COREWEBVIEW2_PERMISSION_STATE_ALLOW)?;
                                }
                                Ok(())
                            },
                        )),
                        &mut token,
                    )
                    .ok();

                    // CRM-Integration URLs abfangen
                    let app_for_crm = app_handle.clone();
                    core.add_NewWindowRequested(
                        &webview2_com::NewWindowRequestedEventHandler::create(Box::new(
                            move |_, args| {
                                if let Some(args) = args {
                                    let mut uri_pwstr = windows::core::PWSTR::null();
                                    args.Uri(&mut uri_pwstr)?;
                                    let uri_str = uri_pwstr.to_string().unwrap_or_default();
                                    if uri_str.contains("4dy-client.localhost/incoming") {
                                        args.SetHandled(true)?;
                                        if let Ok(parsed) = url::Url::parse(&uri_str) {
                                            let name = parsed
                                                .query_pairs()
                                                .find(|(k, _)| k == "name")
                                                .map(|(_, v)| v.to_string())
                                                .unwrap_or_else(|| "Unbekannt".to_string());
                                            let number = parsed
                                                .query_pairs()
                                                .find(|(k, _)| k == "number")
                                                .map(|(_, v)| v.to_string())
                                                .unwrap_or_default();

                                            println!(
                                                "[4dy] CRM-Integration Anruf: {} ({})",
                                                name, number
                                            );
                                            show_call_notification(&app_for_crm, &name, &number);
                                            tauri::Emitter::emit(
                                                &app_for_crm,
                                                "incoming-call",
                                                serde_json::json!({
                                                    "name": name, "number": number,
                                                }),
                                            )
                                            .ok();
                                        }
                                    }
                                }
                                Ok(())
                            },
                        )),
                        &mut token,
                    )
                    .ok();
                }
            })
            .ok();
    }

    if config.window.remember_position {
        if let Some(state) = config::load_window_state() {
            if state.maximized {
                window.maximize().ok();
            }
        }
    }

    // Hotkey-Events → Webview
    let webview = window.clone();
    app.listen("hotkey-action", move |event: tauri::Event| {
        let action = event.payload().trim_matches('"');

        // dial_selection: Strg+C simulieren → Clipboard → normalisieren → Dialer → wählen
        if action == "dial_selection" {
            #[cfg(windows)]
            {
                use windows::Win32::Foundation::HWND;
                use windows::Win32::UI::Input::KeyboardAndMouse::*;

                // 1) Ctrl+C simulieren um Markierung zu kopieren
                unsafe {
                    let inputs = [
                        INPUT {
                            r#type: INPUT_KEYBOARD,
                            Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VK_CONTROL, ..Default::default() } },
                        },
                        INPUT {
                            r#type: INPUT_KEYBOARD,
                            Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VK_C, ..Default::default() } },
                        },
                        INPUT {
                            r#type: INPUT_KEYBOARD,
                            Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VK_C, dwFlags: KEYEVENTF_KEYUP, ..Default::default() } },
                        },
                        INPUT {
                            r#type: INPUT_KEYBOARD,
                            Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VK_CONTROL, dwFlags: KEYEVENTF_KEYUP, ..Default::default() } },
                        },
                    ];
                    SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
                }

                // 2) Kurz warten, dann Clipboard lesen
                let webview_clone = webview.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(150));

                    // Clipboard lesen via Win32
                    let clipboard_text = unsafe {
                        use windows::Win32::Foundation::{HANDLE, HGLOBAL};
                        use windows::Win32::System::DataExchange::*;
                        use windows::Win32::System::Memory::*;
                        let mut text = String::new();
                        if OpenClipboard(Some(HWND::default())).is_ok() {
                            if let Ok(handle) = GetClipboardData(13) { // CF_UNICODETEXT
                                let hglobal = std::mem::transmute::<HANDLE, HGLOBAL>(handle);
                                let ptr = GlobalLock(hglobal) as *const u16;
                                if !ptr.is_null() {
                                    let mut len = 0;
                                    while *ptr.add(len) != 0 { len += 1; }
                                    text = String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len));
                                    GlobalUnlock(hglobal).ok();
                                }
                            }
                            let _ = CloseClipboard();
                        }
                        text
                    };

                    if clipboard_text.is_empty() {
                        return;
                    }

                    // 3) Nummer normalisieren
                    let number = match crate::clipboard::extract_phone_number(&clipboard_text) {
                        Some(n) => n,
                        None => return,
                    };

                    println!("[4dy] Markierte Nummer: '{}' → '{}'", clipboard_text.trim(), number);

                    // 4) Fenster in Vordergrund (thread-safe über Tauri API)
                    webview_clone.show().ok();
                    webview_clone.unminimize().ok();
                    webview_clone.set_focus().ok();

                    let js = format!(
                        "var isOpen = !!document.querySelector('.openContainer'); \
                         if (!isOpen) {{ var btn = document.getElementById('menuDialer'); if (btn) btn.click(); }} \
                         setTimeout(function() {{ \
                             var input = document.getElementById('dialpad-input'); \
                             if (input) {{ \
                                 var nativeSetter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set; \
                                 nativeSetter.call(input, '{}'); \
                                 input.dispatchEvent(new Event('input', {{bubbles: true}})); \
                                 input.focus(); \
                             }} \
                         }}, isOpen ? 50 : 400);",
                        number
                    );
                    webview_clone.eval(&js).ok();
                });
            }
            return;
        }

        // open_dialer: Fenster in Vordergrund + Dialer öffnen
        if action == "open_dialer" {
            #[cfg(windows)]
            {
            }
            webview.show().ok();
            webview.unminimize().ok();
            webview.set_focus().ok();
            webview.eval(
                "var isOpen = !!document.querySelector('.openContainer'); \
                 if (!isOpen) { var btn = document.getElementById('menuDialer'); if (btn) btn.click(); } \
                 setTimeout(function() { \
                     var input = document.getElementById('dialpad-input'); \
                     if (input) { input.focus(); input.select(); } \
                 }, isOpen ? 50 : 300);"
            ).ok();
            return;
        }

        let js = match action {
            "dial_clipboard" => {
                "navigator.clipboard.readText().then(text => { \
                    if (text && window.__4DY_DIAL) { \
                        const cleaned = text.replace(/[^\\d+]/g, ''); \
                        if (cleaned.length >= 3) window.__4DY_DIAL(cleaned); \
                    } \
                }).catch(() => console.warn('[4dy] Clipboard nicht lesbar'));".to_string()
            }
            "answer_call" => "window.__4DY_ANSWER && window.__4DY_ANSWER();".to_string(),
            "hangup" => "window.__4DY_HANGUP && window.__4DY_HANGUP();".to_string(),
            _ => return,
        };
        webview.eval(&js).ok();
    });

    // Fenster-Events
    let window_clone = window.clone();
    window.on_window_event(move |event| {
        match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                window_clone.hide().ok();
            }
            tauri::WindowEvent::Moved(pos) => {
                if let Ok(size) = window_clone.inner_size() {
                    let maximized = window_clone.is_maximized().unwrap_or(false);
                    if !maximized {
                        config::save_window_state(&WindowState {
                            x: pos.x,
                            y: pos.y,
                            width: size.width,
                            height: size.height,
                            maximized,
                        })
                        .ok();
                    }
                }
            }
            tauri::WindowEvent::Resized(size) => {
                if let Ok(pos) = window_clone.outer_position() {
                    let maximized = window_clone.is_maximized().unwrap_or(false);
                    // Nicht speichern wenn maximiert oder wenn es die kompakte Dialer-Größe ist
                    if !maximized && size.width > 780 {
                        config::save_window_state(&WindowState {
                            x: pos.x,
                            y: pos.y,
                            width: size.width,
                            height: size.height,
                            maximized,
                        })
                        .ok();
                    }
                }
            }
            _ => {}
        }
    });

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    fourdy_i18n::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_initial_config,
            clean_phone_number,
            save_window_state,
            on_incoming_call,
            show_notification,
            register_tel_handler,
            dial_number,

            toggle_devtools,

        ])
        .setup(|app| {
            let handle = app.handle().clone();

            if let Err(e) = tray::setup_tray(&handle) {
                eprintln!("Tray-Setup fehlgeschlagen: {}", e);
            }

            if let Ok(config) = config::load_config() {
                    if let Err(e) = hotkeys::register_hotkeys(&handle, &config.hotkeys) {
                        eprintln!("Hotkey-Registrierung fehlgeschlagen: {}", e);
                    }

                    if let Some(main_window) = app.get_webview_window("main") {
                        main_window.hide().ok();
                    }

                    if let Err(e) = create_webclient_window(&handle, &config) {
                        eprintln!("Web-Client-Fenster fehlgeschlagen: {}", e);
                        if let Some(main_window) = app.get_webview_window("main") {
                            main_window.show().ok();
                        }
                    }

                    // dial.txt überwachen (geschrieben von tel-handler.exe)
                    let handle_for_dial = handle.clone();
                    std::thread::spawn(move || {
                        let dial_file = config::config_path()
                            .parent().unwrap().join("dial.txt");
                        loop {
                            std::thread::sleep(std::time::Duration::from_millis(500));
                            if dial_file.exists() {
                                if let Ok(number) = std::fs::read_to_string(&dial_file) {
                                    let number = number.trim().to_string();
                                    if !number.is_empty() {
                                        std::fs::remove_file(&dial_file).ok();
                                        println!("[4dy] tel: Nummer empfangen: {}", number);
                                        if let Some(window) = handle_for_dial.get_webview_window("webclient") {
                                            window.show().ok();
                                            window.unminimize().ok();
                                            window.set_focus().ok();
                                            let js = format!(
                                                "var isOpen = !!document.querySelector('.openContainer'); \
                                                 if (!isOpen) {{ var btn = document.getElementById('menuDialer'); if (btn) btn.click(); }} \
                                                 setTimeout(function() {{ \
                                                     var input = document.getElementById('dialpad-input'); \
                                                     if (input) {{ \
                                                         var nativeSetter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set; \
                                                         nativeSetter.call(input, '{}'); \
                                                         input.dispatchEvent(new Event('input', {{bubbles: true}})); \
                                                         input.focus(); \
                                                     }} \
                                                 }}, isOpen ? 50 : 400);",
                                                number
                                            );
                                            window.eval(&js).ok();
                                        }
                                    }
                                }
                            }
                        }
                    });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Fehler beim Starten der Anwendung");
}
