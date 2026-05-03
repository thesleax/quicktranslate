use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
struct MyMemoryResponse {
    #[serde(rename = "responseData")]
    response_data: ResponseData,
    #[serde(rename = "responseStatus")]
    response_status: i32,
    #[serde(rename = "responseDetails", default)]
    response_details: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseData {
    #[serde(rename = "translatedText")]
    translated_text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeepLResponse {
    translations: Vec<DeepLTranslation>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeepLTranslation {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterResponse {
    choices: Vec<OpenRouterChoice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterChoice {
    message: OpenRouterMessage,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterMessage {
    content: String,
}

pub fn translate_text_sync(
    text: &str,
    source_lang: &str,
    target_lang: &str,
    api_provider: &str,
    api_key: &str,
    translation_type: &str,
) -> Result<String, String> {
    if text.trim().is_empty() {
        return Err("Çevrilecek metin boş".to_string());
    }

    match api_provider {
        "deepL" => translate_deepL(text, source_lang, target_lang, api_key, translation_type),
        "openrouter" => translate_openrouter(text, source_lang, target_lang, api_key, translation_type),
        _ => translate_mymemory(text, source_lang, target_lang),
    }
}

fn translate_mymemory(
    text: &str,
    source_lang: &str,
    target_lang: &str,
) -> Result<String, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let source = if source_lang == "auto" || source_lang.is_empty() {
        "autodetect".to_string()
    } else {
        source_lang.to_string()
    };

    let lang_pair = format!("{}|{}", source, target_lang);
    let encoded_text = urlencoding::encode(text);
    let url = format!(
        "https://api.mymemory.translated.net/get?q={}&langpair={}",
        encoded_text, lang_pair
    );

    let response = client
        .get(&url)
        .send()
        .map_err(|e| format!("İstek hatası: {}", e))?;

    let status = response.status();
    let body = response.text().unwrap_or_default();

    if !status.is_success() {
        return Err(format!("API hatası ({}): {}", status, body));
    }

    if body.contains("\"responseStatus\"") {
        let result: MyMemoryResponse = serde_json::from_str(&body)
            .map_err(|e| format!("JSON parse hatası: {} - Body: {}", e, body))?;

        if result.response_status != 200 {
            return Err(format!("Çeviri API hatası ({}): {:?}", result.response_status, result.response_details));
        }

        Ok(result.response_data.translated_text)
    } else {
        Err(format!("Geçersiz API yanıtı: {}", body))
    }
}

fn translate_deepL(
    text: &str,
    source_lang: &str,
    target_lang: &str,
    api_key: &str,
    translation_type: &str,
) -> Result<String, String> {
    if api_key.is_empty() {
        return Err("DeepL API anahtarı gerekli".to_string());
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;

    let target = if target_lang == "en" {
        "EN-US".to_string()
    } else {
        target_lang.to_uppercase()
    };

    let formality = if translation_type == "formal" { "prefer_more" } else { "prefer_less" };

    let params = [
        ("text", text),
        ("source_lang", source_lang),
        ("target_lang", &target),
        ("formality", formality),
    ];

    let response = client
        .post("https://api-free.deepl.com/v2/translate")
        .header("Authorization", format!("DeepL-Auth-Key {}", api_key))
        .form(&params)
        .send()
        .map_err(|e| format!("DeepL istek hatası: {}", e))?;

    let status = response.status();
    let body = response.text().unwrap_or_default();

    if !status.is_success() {
        return Err(format!("DeepL API hatası ({}): {}", status, body));
    }

    let result: DeepLResponse = serde_json::from_str(&body)
        .map_err(|e| format!("DeepL JSON parse hatası: {} - Body: {}", e, body))?;

    result.translations
        .first()
        .map(|t| t.text.clone())
        .ok_or_else(|| "DeepL yanıtı boş".to_string())
}

fn translate_openrouter(
    text: &str,
    source_lang: &str,
    target_lang: &str,
    api_key: &str,
    translation_type: &str,
) -> Result<String, String> {
    if api_key.is_empty() {
        return Err("OpenRouter API anahtarı gerekli".to_string());
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let style_instruction = if translation_type == "formal" {
        "Use formal, professional language suitable for business correspondence."
    } else {
        "Use casual, conversational language suitable for everyday communication."
    };

    let source_name = lang_code_to_name(source_lang);
    let target_name = lang_code_to_name(target_lang);

    let prompt = format!(
        "Translate the following text from {} to {}. {}\n\nText to translate:\n{}",
        source_name, target_name, style_instruction, text
    );

    let request_body = serde_json::json!({
        "model": "openai/gpt-3.5-turbo",
        "messages": [
            {"role": "user", "content": prompt}
        ],
        "max_tokens": 2000
    });

    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .map_err(|e| format!("OpenRouter istek hatası: {}", e))?;

    let status = response.status();
    let body = response.text().unwrap_or_default();

    if !status.is_success() {
        return Err(format!("OpenRouter API hatası ({}): {}", status, body));
    }

    let result: OpenRouterResponse = serde_json::from_str(&body)
        .map_err(|e| format!("OpenRouter JSON parse hatası: {} - Body: {}", e, body))?;

    result.choices
        .first()
        .map(|c| c.message.content.trim().to_string())
        .ok_or_else(|| "OpenRouter yanıtı boş".to_string())
}

fn lang_code_to_name(code: &str) -> &str {
    match code {
        "auto" | "autodetect" => "the detected language",
        "tr" => "Turkish",
        "en" => "English",
        "de" => "German",
        "fr" => "French",
        "es" => "Spanish",
        "it" => "Italian",
        "pt" => "Portuguese",
        "ru" => "Russian",
        "ar" => "Arabic",
        "ja" => "Japanese",
        "ko" => "Korean",
        "zh" => "Chinese",
        "hi" => "Hindi",
        "nl" => "Dutch",
        "pl" => "Polish",
        "sv" => "Swedish",
        "da" => "Danish",
        "no" => "Norwegian",
        "fi" => "Finnish",
        "el" => "Greek",
        "he" => "Hebrew",
        "th" => "Thai",
        "vi" => "Vietnamese",
        "id" => "Indonesian",
        "ms" => "Malay",
        _ => code,
    }
}

mod urlencoding {
    pub fn encode(input: &str) -> String {
        let mut encoded = String::new();
        for byte in input.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    encoded.push(byte as char);
                }
                b' ' => encoded.push_str("%20"),
                _ => {
                    encoded.push_str(&format!("%{:02X}", byte));
                }
            }
        }
        encoded
    }
}