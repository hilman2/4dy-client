#![cfg_attr(not(test), windows_subsystem = "windows")]

mod phone;

use phone::normalize_phone;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    fourdy_i18n::init();

    let args: Vec<String> = env::args().collect();

    let tel_arg = args.get(1).cloned().unwrap_or_default();
    let raw = tel_arg
        .trim_start_matches("tel:")
        .trim_start_matches("//")
        .trim();

    if raw.is_empty() {
        return;
    }

    let number = match normalize_phone(raw) {
        Some(n) => n,
        None => return,
    };

    // Nummer in %APPDATA%/4dy-client/dial.txt schreiben
    // Die Tauri-App beobachtet diese Datei und wählt die Nummer
    let appdata = env::var("APPDATA").unwrap_or_else(|_| ".".into());
    let dial_file = PathBuf::from(&appdata).join("4dy-client").join("dial.txt");

    fs::create_dir_all(dial_file.parent().unwrap()).ok();
    fs::write(&dial_file, &number).ok();
}
