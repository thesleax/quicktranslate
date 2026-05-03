use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub source_lang: String,
    pub target_lang: String,
    pub auto_start: bool,
    pub hotkey: String,
    pub setup_completed: bool,
    pub translation_type: String, // "formal" | "casual"
    pub api_provider: String,     // "mymemory" | "deepL" | "openrouter"
    pub api_key: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            source_lang: "en".to_string(),
            target_lang: "tr".to_string(),
            auto_start: false,
            hotkey: "Ctrl+Shift+T".to_string(),
            setup_completed: false,
            translation_type: "casual".to_string(),
            api_provider: "mymemory".to_string(),
            api_key: String::new(),
        }
    }
}

pub static SETTINGS: Lazy<RwLock<Settings>> = Lazy::new(|| {
    RwLock::new(load_settings_from_file())
});

pub static TRANSLATION_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

fn get_settings_path() -> std::path::PathBuf {
    let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(app_data).join("QuickTranslate").join("settings.json")
}

fn load_settings_from_file() -> Settings {
    let path = get_settings_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str(&content) {
                return settings;
            }
        }
    }
    Settings::default()
}

pub fn save_settings_to_file(settings: &Settings) -> Result<(), String> {
    let path = get_settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_supported_langs() -> HashMap<String, String> {
    let mut langs = HashMap::new();
    langs.insert("auto".to_string(), "Otomatik".to_string());
    langs.insert("tr".to_string(), "Türkçe".to_string());
    langs.insert("en".to_string(), "İngilizce".to_string());
    langs.insert("de".to_string(), "Almanca".to_string());
    langs.insert("fr".to_string(), "Fransızca".to_string());
    langs.insert("es".to_string(), "İspanyolca".to_string());
    langs.insert("it".to_string(), "İtalyanca".to_string());
    langs.insert("pt".to_string(), "Portekizce".to_string());
    langs.insert("ru".to_string(), "Rusça".to_string());
    langs.insert("ar".to_string(), "Arapça".to_string());
    langs.insert("ja".to_string(), "Japonca".to_string());
    langs.insert("ko".to_string(), "Korece".to_string());
    langs.insert("zh".to_string(), "Çince".to_string());
    langs.insert("hi".to_string(), "Hintçe".to_string());
    langs.insert("nl".to_string(), "Felemenkçe".to_string());
    langs.insert("pl".to_string(), "Lehçe".to_string());
    langs.insert("sv".to_string(), "İsveççe".to_string());
    langs.insert("da".to_string(), "Danca".to_string());
    langs.insert("no".to_string(), "Norveççe".to_string());
    langs.insert("fi".to_string(), "Fince".to_string());
    langs.insert("el".to_string(), "Yunanca".to_string());
    langs.insert("he".to_string(), "İbranice".to_string());
    langs.insert("th".to_string(), "Tayca".to_string());
    langs.insert("vi".to_string(), "Vietnamca".to_string());
    langs.insert("id".to_string(), "Endonezce".to_string());
    langs.insert("ms".to_string(), "Malezyaca".to_string());
    langs
}