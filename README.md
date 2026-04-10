# Tempest Type (Cross-Platform Rust Native)

Tempest Type is an ultra-fast, entirely local voice-to-text dictation app for macOS and Linux. It runs passively in your menu bar and allows you to instantly transcribe audio anywhere on your system using a global Push-to-Talk hotkey.

This repository features a native Rust core, offering **virtually zero CPU overhead** while idling, native Apple Silicon GPU acceleration (via coreML/Metal `whisper.cpp`) on macOS, and efficient local CPU/GPU transcription on Linux.

## Features
* 🎙️ **Push-to-Talk**: Hold down **Right Option (⌥)** on macOS or **AltGr** on Linux to record anywhere. Let go, and the text is automatically typed at your cursor.
* ⚡ **Ultra-Fast & Local**: Uses `whisper-rs` (a wrapper around `whisper.cpp`) to transcribe audio locally on your machine in milliseconds.
* 🧠 **AI Grammar Cleanup**: Automatically pipes the raw transcription through a local Ollama instance (`qwen2.5-coder:3b`) to fix grammar, punctuation, and speech-to-text hallucinations.
* 📝 **Meeting Memos**: A secondary mode to summarize long recordings into clear Markdown bullet points and action items.
* 🔔 **Native System Integration**: Cross-platform system sounds and system-native menu bar/system tray icon.

## Installation & Usage

You can use the provided `install.sh` script to handle dependencies and setup auto-start, or compile it manually.

### Prerequisites (Automatic via `install.sh`):
- **macOS**: Homebrew, CMake, Ollama.
- **Linux**: Build-essential, CMake, ALSA, X11, GTK3, and AppIndicator libraries.

### Quick Start:

1. Clone the repository:
   ```bash
   git clone https://github.com/7empest462/TempestType.git
   cd TempestType
   ```

2. Run the installer:
   ```bash
   ./install.sh
   ```

3. **Permissions (Crucial Step)**:
   - **macOS**: Grant **Accessibility**, **Input Monitoring**, and **Microphone** permissions to the binary (`~/.tempest-type/tempest-type`) in System Settings.
   - **Linux (X11)**: Ensure your user is part of the `input` group if using low-level hotkeys (e.g., `sudo usermod -aG input $USER`). Note that hotkeys work best on X11; Wayland support may require additional configuration.

### Configuration (Customizing Hotkeys)
The first time you run Tempest Type, it will create a default configuration file at `~/.tempest-type/config.json`. You can edit this file to change your Push-to-Talk hotkey or the AI model used for cleanup.

Example `config.json`:
```json
{
  "hotkey": "AltGr",
  "model": "qwen2.5-coder:3b"
}
```

Supported Hotkeys:
-   `AltGr` (Standard for Right Option/Right Alt)
-   `CapsLock`, `ControlLeft`, `ControlRight`, `ShiftLeft`, `ShiftRight`, `MetaLeft` (Cmd/Win)
-   `F1` through `F12`
-   `Space`, `Tab`

### Manual Compilation:
```bash
cd tempest-type
cargo build --release
```
Then run the binary: `./target/release/tempest-type`.

---

*Say goodbye to cloud subscriptions and slow web-based dictation!*
