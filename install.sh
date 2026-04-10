#!/bin/bash
# Tempest Type Installer (Cross-platform Rust Native Version)
# Installs Tempest Type - a local voice-to-text app for macOS and Linux
# Requirements: macOS with Apple Silicon or Linux with GTK3/X11/ALSA

set -e

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║          Tempest Type Installer (Cross-platform)          ║"
echo "║     Local AI Voice-to-Text with Push-to-Talk (Native)     ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Configuration
INSTALL_DIR="$HOME/.tempest-type"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RUST_PROJ_DIR="$SCRIPT_DIR/tempest-type"

# Migration logic
OLD_DIR="$HOME/.voice-type"
if [ -d "$OLD_DIR" ] && [ ! -d "$INSTALL_DIR" ]; then
    echo "🔄 Migrating configuration from $OLD_DIR to $INSTALL_DIR..."
    mv "$OLD_DIR" "$INSTALL_DIR"
fi

OS_TYPE=$(uname -s)

install_linux_deps() {
    echo "📦 Detected Linux. Installing system dependencies..."
    if command -v apt-get &> /dev/null; then
        sudo apt-get update
        sudo apt-get install -y build-essential cmake pkg-config libasound2-dev libx11-dev libxtst-dev libxdo-dev \
            libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libgtk-3-dev libayatana-appindicator3-dev \
            libdbus-1-dev libxcursor-dev libxinerama-dev libxi-dev libxrandr-dev libx11-xcb-dev \
            x11proto-dev libxext-dev libice-dev libsm-dev
    elif command -v dnf &> /dev/null; then
        sudo dnf install -y gcc-c++ cmake pkgconf-pkg-config alsa-lib-devel libX11-devel libXtst-devel libxdo-devel \
            libxcb-devel libxkbcommon-devel gtk3-devel libappindicator-gtk3-devel dbus-devel \
            libXcursor-devel libXinerama-devel libXi-devel libXrandr-devel \
            xorg-x11-proto-devel libXext-devel libICE-devel libSM-devel
    elif command -v pacman &> /dev/null; then
        sudo pacman -S --needed base-devel cmake pkgconf alsa-lib libx11 libxtst libxdo libxcb libxkbcommon \
            gtk3 libappindicator-gtk3 dbus libxcursor libxinerama libxi libxrandr \
            xorgproto libxext libice libsm
    else
        echo "❌ Unsupported package manager. Please install dependencies manually."
        exit 1
    fi
}

check_linux_libs() {
    echo "🔍 Checking for required libraries via pkg-config..."
    # Force common Linux paths just in case the environment is clean
    export PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/share/pkgconfig:/usr/lib/pkgconfig:$PKG_CONFIG_PATH"
    
    MISSING=0
    # Use ayatana-appindicator3-0.1 or appindicator3-0.1 as fallbacks
    REQS=("x11" "xtst" "gtk+-3.0")
    
    # Check core X11/GTK first
    for req in "${REQS[@]}"; do
        if ! pkg-config --print-errors --exists "$req"; then
            echo "❌ Missing dependency: $req"
            MISSING=1
        else
            echo "✓ Found: $req ($(pkg-config --modversion $req))"
        fi
    done

    # Check AppIndicator separately with fallbacks
    if ! pkg-config --exists "ayatana-appindicator3-0.1" && ! pkg-config --exists "appindicator3-0.1"; then
        echo "❌ Missing dependency: ayatana-appindicator3-0.1 (or appindicator3-0.1)"
        MISSING=1
    else
        echo "✓ Found: AppIndicator"
    fi

    if [ "$MISSING" == "1" ]; then
        echo "❌ Some dependencies are still missing. Please check the error messages above."
        exit 1
    fi
}

install_macos_deps() {
    echo "🍎 Detected macOS. Checking for Homebrew dependencies..."
    if ! command -v brew &> /dev/null; then
        echo "❌ Homebrew not found. Please install Homebrew first."
        exit 1
    fi
    brew install cmake ollama
}

# 1. Install Dependencies
if [ "$OS_TYPE" == "Darwin" ]; then
    install_macos_deps
elif [ "$OS_TYPE" == "Linux" ]; then
    install_linux_deps
else
    echo "❌ Unsupported OS: $OS_TYPE"
    exit 1
fi

# 2. Check for Ollama
if ! command -v ollama &> /dev/null; then
    echo "⚠️  Ollama not found. Please install it from https://ollama.com"
fi

# 3. Check for Rust (Cargo)
if ! command -v cargo &> /dev/null; then
    echo "⚠️  Rust (cargo) not found. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi
echo "✓ Rust (cargo) found"

# 3.5 Check Linux Libs
if [ "$OS_TYPE" == "Linux" ]; then
    check_linux_libs
fi

# 4. Build the Rust Project
echo ""
echo "🦀 Building Tempest Type Rust application..."
cd "$RUST_PROJ_DIR"
cargo build --release
echo "✓ Build successful"

# 5. Create install directory and copy binary
echo ""
echo "📁 Installing to $INSTALL_DIR..."
mkdir -p "$INSTALL_DIR"

if [ "$OS_TYPE" == "Darwin" ]; then
    echo "🏗️  Installing to $INSTALL_DIR (Raw Binary Mode)"
    
    # Cleanup old bundle if exists
    rm -rf "/Applications/Tempest Type.app" 2>/dev/null || true
    rm -rf "$INSTALL_DIR/Tempest Type.app" 2>/dev/null || true
    
    cp "$RUST_PROJ_DIR/target/release/tempest-type" "$INSTALL_DIR/tempest-type"
    cp "$RUST_PROJ_DIR/icon.png" "$INSTALL_DIR/icon.png" 2>/dev/null || true
    
    # Critical for macOS Apple Silicon (prevent Killed: 9)
    echo "🔐 Signing binary..."
    xattr -rc "$INSTALL_DIR/tempest-type" 2>/dev/null || true
    codesign --force -s - "$INSTALL_DIR/tempest-type" 2>/dev/null || true
    
    echo "✓ Installed and signed binary to $INSTALL_DIR/tempest-type"
else
    cp "$RUST_PROJ_DIR/target/release/tempest-type" "$INSTALL_DIR/tempest-type"
    cp "$RUST_PROJ_DIR/icon.png" "$INSTALL_DIR/icon.png" 2>/dev/null || true
    echo "✓ Successfully installed binary and assets"
fi

# 6. Setup Auto-start
if [ "$OS_TYPE" == "Darwin" ]; then
    PLIST_PATH="$HOME/Library/LaunchAgents/com.user.tempest-type.plist"
    cat > "$PLIST_PATH" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.user.tempest-type</string>
    <key>ProgramArguments</key>
    <array>
        <string>$INSTALL_DIR/tempest-type</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
    <key>StandardOutPath</key>
    <string>/tmp/tempest-type.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/tempest-type.err</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:$HOME/.cargo/bin</string>
    </dict>
</dict>
</plist>
EOF
    echo "🚀 Loading LaunchAgent..."
    echo "✓ Setup macOS auto-start (LaunchAgent created but not started)"
    
    # Create Desktop command for macOS
    echo "💡 Creating Desktop Command script..."
    CMD_FILE="$HOME/Desktop/Tempest Type.command"
    cat > "$CMD_FILE" << EOF
#!/bin/bash
clear
"$INSTALL_DIR/tempest-type"
EOF
    chmod +x "$CMD_FILE"
    echo "✓ Created macOS desktop command script"
    
    # Cleanup old Alias if exists
    rm "$HOME/Desktop/Tempest Type" 2>/dev/null || true

elif [ "$OS_TYPE" == "Linux" ]; then
    AUTOSTART_DIR="$HOME/.config/autostart"
    mkdir -p "$AUTOSTART_DIR"
    DESKTOP_FILE="$AUTOSTART_DIR/tempest-type.desktop"
    cat > "$DESKTOP_FILE" << EOF
[Desktop Entry]
Type=Application
Name=Tempest Type
Comment=Local AI Voice-to-Text
Exec=$INSTALL_DIR/tempest-type
Icon=$INSTALL_DIR/icon.png
Terminal=false
X-GNOME-Autostart-enabled=true
EOF
    chmod +x "$DESKTOP_FILE"
    echo "✓ Setup Linux auto-start (.desktop file created but not started)"

    # Create Desktop shortcut for Linux
    DESKTOP_HOME_FILE="$HOME/Desktop/tempest-type.desktop"
    cp "$DESKTOP_FILE" "$DESKTOP_HOME_FILE"
    chmod +x "$DESKTOP_HOME_FILE"
    echo "✓ Created Linux desktop shortcut"
fi

# 7. Pull Ollama model
echo ""
echo "🧠 Pulling Ollama model (qwen2.5:3b)..."
ollama pull qwen2.5:3b || echo "⚠️  Could not pull model automatically. Ensure Ollama is running."

echo ""
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║                 Installation Complete!                    ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""
echo "🎤 HOW TO USE:"
echo "   Hold RIGHT OPTION (macOS) or AltGr (Linux) to record voice."
echo "   Release to type transcribed text!"
echo ""
echo "📋 Logs: tail -f /tmp/tempest-type.log"
echo ""
