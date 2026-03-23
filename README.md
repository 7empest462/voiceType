# Voice Type (Rust Native for macOS)

Voice Type is an ultra-fast, entirely local voice-to-text dictation app for macOS. It runs passively in your menu bar and allows you to instantly transcribe audio anywhere on your system using a global Push-to-Talk hotkey.

This repository features the native Rust rewrite, offering **virtually zero CPU overhead** while idling, native Apple Silicon GPU acceleration (via coreML/Metal `whisper.cpp`), and lightning-fast dictation cleanup using local LLMs. 

## Features
* 🎙️ **Push-to-Talk**: Hold down the **Right Option (⌥)** key to record anywhere. Let go, and the text is automatically typed at your cursor.
* ⚡ **Ultra-Fast & Local**: Uses `whisper-rs` (a wrapper around `whisper.cpp`) to transcribe audio locally on your Mac's Neural Engine / GPU in milliseconds. No cloud APIs, no subscriptions.
* 🧠 **AI Grammar Cleanup**: Automatically pipes the raw transcription through a local Ollama instance (`qwen2.5-coder:7b`) to fix grammar, punctuation, and speech-to-text hallucinations.
* 📝 **Meeting Memos**: A secondary mode to summarize long recordings into clear Markdown bullet points and action items, instantly saving to your Documents folder.
* 🔔 **Native macOS Integration**: Menu bar icon, native `afplay` sound effects, and `rdev` low-level input injection.

## Installation & Usage

You do not need our provided installer to run this natively. Anyone can compile and use it straight from source!

### Prerequisites:
1. **Rust & Cargo**: (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
2. **CMake**: (`brew install cmake`) - required for building `whisper.cpp`
3. **Ollama**: (`brew install ollama`) - required for the AI grammar cleanup step.

### Quick Start:

1. Clone the repository and navigate to the Rust project:
   ```bash
   git clone https://github.com/7empest462/VoiceType.git
   cd VoiceType/voice_type_rs
   ```

2. Compile the binary in release mode:
   ```bash
   cargo build --release
   ```

3. Start the local cleanup LLM in a separate terminal:
   ```bash
   ollama run qwen2.5-coder:7b
   ```

4. Run the executable:
   ```bash
   cargo run --release
   ```
   *(Note: The first time you run the app, it will download the ~480MB Whisper model `ggml-small.en.bin` to `~/.voice-type/models/`).*

### Permissions (Crucial Step)
Because Voice Type relies on listening to global keyboard events and injecting text natively, macOS will block it by default due to strict security policies.

When you run the binary (or the `LaunchAgent` background service), you **must** manually grant permissions:
1. Open **System Settings > Privacy & Security**.
2. Go to **Accessibility** and ensure the toggle for your Terminal (e.g., `iTerm2` or `Terminal`) OR the native binary (`~/.voice-type/voice_type_rs`) is switched **ON**.
3. Go to **Input Monitoring** and do the **exact same thing**.
4. Go to **Microphone** and ensure it has recording access.

> **💡 Troubleshooting Stale Permissions**: If you compiled a *new* Rust binary or ran the `install.sh` sequence, macOS will silently block the new executable even if the toggle looks checked. If your hotkey isn't responding:
> 1. Select the `voice_type_rs` entry in Accessibility/Input Monitoring and click the **minus (`-`) button** to delete it.
> 2. Click the **plus (`+`) button**.
> 3. Press `Cmd + Shift + G` to open the "Go to Folder" window, paste `~/.voice-type`, and select the `voice_type_rs` executable.
> 4. Ensure the toggles are ON, and restart the background daemon.

### Auto-Start Service (LaunchAgent)
If you'd like to run it silently in the background on startup, you can use the `install.sh` script included in the root directory, which will compile the binary, move it to `~/.voice-type/`, and enroll a `.plist` daemon in `~/Library/LaunchAgents`.

---

*Say goodbye to cloud subscriptions and slow web-based dictation!*
