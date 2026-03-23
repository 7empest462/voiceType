#!/bin/bash
# Voice Type Installer (Rust Native Version)
# Installs Voice Type - a local voice-to-text app for macOS
# Requirements: macOS with Apple Silicon, Homebrew

set -e

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║           Voice Type Installer for macOS                  ║"
echo "║     Local AI Voice-to-Text with Push-to-Talk (Native)     ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Configuration
INSTALL_DIR="$HOME/.voice-type"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RUST_PROJ_DIR="$SCRIPT_DIR/voice_type_rs"
PLIST_PATH="$HOME/Library/LaunchAgents/com.user.voice-type.plist"

# 1. Check for Homebrew
if ! command -v brew &> /dev/null; then
    echo "❌ Homebrew not found. Please install Homebrew first:"
    echo "   /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
    exit 1
fi
echo "✓ Homebrew found"

# 2. Check for Ollama
if ! command -v ollama &> /dev/null; then
    echo "⚠️  Ollama not found. Installing via Homebrew..."
    brew install ollama
fi
echo "✓ Ollama found"

# 3. Check for Rust (Cargo)
if ! command -v cargo &> /dev/null; then
    echo "⚠️  Rust (cargo) not found. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi
echo "✓ Rust (cargo) found"

# 4. Check for CMake (required for whisper.cpp building)
if ! command -v cmake &> /dev/null; then
    echo "⚠️  CMake not found. Installing via Homebrew..."
    brew install cmake
fi
echo "✓ CMake found"

# 5. Build the Rust Project
echo ""
echo "🦀 Building Voice Type Rust application (this may take a couple minutes)..."
cd "$RUST_PROJ_DIR"
cargo build --release
echo "✓ Build successful"

# 6. Create install directory and copy binary
echo ""
echo "📁 Installing to $INSTALL_DIR..."
mkdir -p "$INSTALL_DIR"
cp "$RUST_PROJ_DIR/target/release/voice_type_rs" "$INSTALL_DIR/voice_type_rs"
echo "✓ Successfully installed binary"

# 7. Create LaunchAgent plist
cat > "$PLIST_PATH" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.user.voice-type</string>
    
    <key>ProgramArguments</key>
    <array>
        <string>$INSTALL_DIR/voice_type_rs</string>
    </array>
    
    <key>RunAtLoad</key>
    <true/>
    
    <key>KeepAlive</key>
    <false/>
    
    <key>StandardOutPath</key>
    <string>/tmp/voice-type.log</string>
    
    <key>StandardErrorPath</key>
    <string>/tmp/voice-type.err</string>
    
    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:$HOME/.cargo/bin</string>
    </dict>
</dict>
</plist>
EOF
echo "✓ Created LaunchAgent (auto-start on login)"

# 8. Pull Ollama model
echo ""
echo "🧠 Pulling Ollama model for AI cleanup (qwen2.5-coder:7b)..."
ollama pull qwen2.5-coder:7b || echo "⚠️  Could not pull model automatically. Please start Ollama first: ollama serve"

# 9. Load LaunchAgent
echo ""
echo "🚀 Starting Voice Type..."
launchctl unload "$PLIST_PATH" 2>/dev/null || true
launchctl load "$PLIST_PATH"

echo ""
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║                 Installation Complete!                    ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""
echo "📍 Installed to: $INSTALL_DIR"
echo ""
echo "🎤 HOW TO USE:"
echo "   Hold RIGHT OPTION (⌥) to record your voice"
echo "   Release to automatically type transcribed/cleaned text!"
echo ""
echo "📝 MEETING MEMO:"
echo "   Click the 🎙️ menu bar icon and select 'Start Meeting Memo'."
echo "   It will summarize your recording and save it to Documents/Memos."
echo ""
echo "⚠️  IMPORTANT: Because this is a native hotkey app, macOS needs permissions."
echo "   Please grant Accessibility and Microphone to 'voice_type_rs' in System Settings."
echo ""
echo "📋 Logs: tail -f /tmp/voice-type.log"
echo ""
