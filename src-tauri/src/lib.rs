mod i18n;
mod settings;
mod translate;

use i18n::Translations;
use settings::{get_supported_langs, save_settings_to_file, SETTINGS, TRANSLATION_IN_PROGRESS};
use std::sync::atomic::Ordering;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WindowEvent,
};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use tauri_plugin_log::{Target, TargetKind};
use enigo::{Enigo, Keyboard, Settings, Key, Direction};

#[tauri::command]
async fn get_settings() -> Result<settings::Settings, String> {
    let settings = SETTINGS.read();
    Ok(settings.clone())
}

#[tauri::command]
async fn update_settings(new_settings: settings::Settings) -> Result<(), String> {
    let mut settings = SETTINGS.write();
    *settings = new_settings.clone();
    save_settings_to_file(&settings)?;
    Ok(())
}

#[tauri::command]
async fn get_languages() -> Result<std::collections::HashMap<String, String>, String> {
    Ok(get_supported_langs())
}

fn send_ctrl_c() {
    if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
        let _ = enigo.key(Key::Control, Direction::Press);
        let _ = enigo.key(Key::Unicode('c'), Direction::Click);
        let _ = enigo.key(Key::Control, Direction::Release);
    }
}

fn send_ctrl_v() {
    if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
        let _ = enigo.key(Key::Control, Direction::Press);
        let _ = enigo.key(Key::Unicode('v'), Direction::Click);
        let _ = enigo.key(Key::Control, Direction::Release);
    }
}

fn send_ctrl_a() {
    if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
        let _ = enigo.key(Key::Control, Direction::Press);
        let _ = enigo.key(Key::Unicode('a'), Direction::Click);
        let _ = enigo.key(Key::Control, Direction::Release);
    }
}

#[tauri::command]
async fn translate_and_replace(app: AppHandle) -> Result<String, String> {
    if TRANSLATION_IN_PROGRESS.load(Ordering::SeqCst) {
        return Err("Çeviri zaten devam ediyor".to_string());
    }

    TRANSLATION_IN_PROGRESS.store(true, Ordering::SeqCst);

    // Step 1: Small delay
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Step 2: Select all first (Ctrl+A) - this works whether something is selected or not
    send_ctrl_a();
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Step 3: Copy selection (Ctrl+C)
    send_ctrl_c();
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Step 4: Read from clipboard
    let mut text = match app.clipboard().read_text() {
        Ok(t) => t,
        Err(e) => {
            TRANSLATION_IN_PROGRESS.store(false, Ordering::SeqCst);
            return Err(format!("Clipboard okunamadı: {}", e));
        }
    };

    if text.trim().is_empty() {
            TRANSLATION_IN_PROGRESS.store(false, Ordering::SeqCst);
            return Err("Çevrilecek metin bulunamadı".to_string());
        }

    let source_lang = SETTINGS.read().source_lang.clone();
    let target_lang = SETTINGS.read().target_lang.clone();
    let api_provider = SETTINGS.read().api_provider.clone();
    let api_key = SETTINGS.read().api_key.clone();
    let translation_type = SETTINGS.read().translation_type.clone();

    // Step 4: Translate
    let translated = match translate::translate_text_sync(&text, &source_lang, &target_lang, &api_provider, &api_key, &translation_type) {
        Ok(t) => t,
        Err(e) => {
            TRANSLATION_IN_PROGRESS.store(false, Ordering::SeqCst);
            return Err(format!("Çeviri hatası: {}", e));
        }
    };

    // Step 5: Write to clipboard
    if let Err(e) = app.clipboard().write_text(&translated) {
        TRANSLATION_IN_PROGRESS.store(false, Ordering::SeqCst);
        return Err(format!("Clipboard yazılamadı: {}", e));
    }

    // Small delay
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Step 6: Select all (Ctrl+A)
    send_ctrl_a();
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Step 7: Paste (Ctrl+V)
    send_ctrl_v();

    TRANSLATION_IN_PROGRESS.store(false, Ordering::SeqCst);
    Ok(translated)
}

#[tauri::command]
async fn translate_text(
    text: String,
    source_lang: String,
    target_lang: String,
    api_provider: String,
    api_key: String,
    translation_type: String,
) -> Result<String, String> {
    if TRANSLATION_IN_PROGRESS.load(Ordering::SeqCst) {
        return Err("Çeviri zaten devam ediyor".to_string());
    }

    TRANSLATION_IN_PROGRESS.store(true, Ordering::SeqCst);
    let result = translate::translate_text_sync(&text, &source_lang, &target_lang, &api_provider, &api_key, &translation_type);
    TRANSLATION_IN_PROGRESS.store(false, Ordering::SeqCst);
    result
}

#[tauri::command]
async fn show_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn hide_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn register_hotkey(app: AppHandle, hotkey: String) -> Result<(), String> {
    let shortcut: Shortcut = hotkey.parse().map_err(|e| format!("{:?}", e))?;

    let app_for_translate = app.clone();
    app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, _event| {
        // Don't emit event, just do translation directly
        let app_clone = app_for_translate.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _ = translate_and_replace(app_clone).await;
            });
        });
    }).map_err(|e| e.to_string())?;

    Ok(())
}

fn setup_tray(app: &AppHandle, i18n: &Translations) -> Result<(), Box<dyn std::error::Error>> {
    let show_i = MenuItem::with_id(app, "show", i18n.t("show"), true, None::<&str>)?;
    let settings_i = MenuItem::with_id(app, "settings", i18n.t("settings"), true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", i18n.t("quit"), true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_i, &settings_i, &quit_i])?;

    let icon_bytes = include_bytes!("../icons/32x32.png");
    let icon = Image::from_bytes(icon_bytes)?;

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip(i18n.t("tooltip"))
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "settings" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = app.emit("open-settings", ());
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
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

fn setup_global_shortcut(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let settings = SETTINGS.read();
    let hotkey = settings.hotkey.clone();

    let shortcut: Shortcut = hotkey.parse()?;
    let app_for_translate = app.clone();

    app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, _event| {
        // Use a separate thread with its own tokio runtime
        let app_clone = app_for_translate.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _ = translate_and_replace(app_clone).await;
            });
        });
    }).map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                ])
                .build(),
        )
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            update_settings,
            get_languages,
            translate_text,
            translate_and_replace,
            show_window,
            hide_window,
            register_hotkey,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let i18n = Translations::new();
            setup_tray(&handle, &i18n)?;
            setup_global_shortcut(&handle)?;

            // Check if launched with --minimized (autostart) or not
            let args: Vec<String> = std::env::args().collect();
            let should_minimize = args.contains(&"--minimized".to_string());

            if let Some(window) = app.get_webview_window("main") {
                // If not minimized (portable run), show window
                if !should_minimize {
                    let _ = window.show();
                    let _ = window.set_focus();
                }

                let w = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = w.hide();
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}