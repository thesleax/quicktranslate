# QuickTranslate

Instant translation app - Translate selected text instantly with a global hotkey and replace it.

![QuickTranslate](https://img.shields.io/badge/Version-1.0.0-blue)
![Platform](https://img.shields.io/badge/Platform-Windows-green)
![License](https://img.shields.io/badge/License-MIT-purple)

## Features

- 🌐 **Runs in System Tray** - Always running in the background
- ⌨️ **Global Hotkey** - Works in any application
- 🔄 **Instant Translation & Replace** - Automatically translates and pastes selected text
- 🌐 **Multi-Language Support** - Translate between 14+ languages
- 📝 **Translation Style** - Formal or Casual options
- 🔑 **Multiple API Support** - MyMemory (free), DeepL, OpenRouter

## Supported Languages

English, Turkish, German, French, Spanish, Italian, Portuguese, Russian, Arabic, Japanese, Korean, Chinese

## Installation

### Requirements

- Windows 10/11
- [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (usually pre-installed)

### Steps

1. Download the latest version from [Releases](https://github.com/thesleax/quicktranslate/releases)
2. Run `QuickTranslate_1.0.0_x64-setup.exe`
3. Follow the installation wizard
4. The app will start automatically when installation is complete

## Usage

### Translating Text

1. Select the text you want to translate (in any application)
2. Press `Ctrl+Shift+T` key combination
3. The selected text is automatically translated and pasted

### Settings

Right-click the system tray icon and select "Settings":

- **Source Language**: Select source language ("Auto" for automatic detection)
- **Target Language**: Select target language
- **Translation Style**: Formal (business) or Casual (everyday)
- **Service**: Select translation service (MyMemory free, DeepL, or OpenRouter)
- **API Key**: Enter API key if required for DeepL or OpenRouter
- **Hotkey**: Change the global hotkey combination
- **Auto Start**: Start automatically when Windows boots

## API Services

### MyMemory (Free)
- Unlimited usage, no API key required
- Quality: Good

### DeepL
- API key required (deepl.com/pro-api)
- Quality: Very Good
- Supports formal/casual style

### OpenRouter
- API key required (openrouter.ai/keys)
- Quality: Very Good (AI-powered translation)
- Natural and fluent translations

## System Requirements

- Windows 10 or later
- 50MB disk space
- WebView2 Runtime

## Contributing

1. Fork the repository
2. Create a new branch (`git checkout -b feature/new-feature`)
3. Commit your changes (`git commit -m 'Add new feature'`)
4. Push to the branch (`git push origin feature/new-feature`)
5. Open a Pull Request

## License

[MIT License](LICENSE)

---

**Note**: If you find a bug or have suggestions, please report them via [Issues](https://github.com/thesleax/quicktranslate/issues).
