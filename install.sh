#!/bin/bash

set -e

echo "📺 Installing tvcode - Apple TV Video Transcoder"
echo "================================================"
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Rust is not installed"
    echo "   Install from: https://rustup.rs/"
    exit 1
fi

# Check if ffmpeg is installed
if ! command -v ffmpeg &> /dev/null || ! command -v ffprobe &> /dev/null; then
    echo "⚠️  Warning: ffmpeg not found"
    echo "   Install with: brew install ffmpeg (macOS)"
    echo ""
fi

# Build the project
echo "🔨 Building tvcode..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Build successful"
echo ""

# Install options
echo "Choose installation method:"
echo "  1) Install to /usr/local/bin (recommended, requires sudo)"
echo "  2) Add to PATH in ~/.zshrc"
echo "  3) Add to PATH in ~/.bashrc"
echo "  4) Skip installation (use from target/release/tvcode)"
echo ""
read -p "Enter choice [1-4]: " choice

case $choice in
    1)
        echo ""
        echo "Installing to /usr/local/bin..."
        sudo cp target/release/tvcode /usr/local/bin/
        echo "✅ Installed to /usr/local/bin/tvcode"
        echo ""
        echo "You can now run: tvcode"
        ;;
    2)
        echo ""
        INSTALL_PATH="/Volumes/Phobos/projects/home_video_loader/target/release"
        if ! grep -q "$INSTALL_PATH" ~/.zshrc 2>/dev/null; then
            echo "export PATH=\"$INSTALL_PATH:\$PATH\"" >> ~/.zshrc
            echo "✅ Added to ~/.zshrc"
            echo ""
            echo "Run: source ~/.zshrc"
            echo "Then: tvcode"
        else
            echo "✅ Already in ~/.zshrc"
        fi
        ;;
    3)
        echo ""
        INSTALL_PATH="/Volumes/Phobos/projects/home_video_loader/target/release"
        if ! grep -q "$INSTALL_PATH" ~/.bashrc 2>/dev/null; then
            echo "export PATH=\"$INSTALL_PATH:\$PATH\"" >> ~/.bashrc
            echo "✅ Added to ~/.bashrc"
            echo ""
            echo "Run: source ~/.bashrc"
            echo "Then: tvcode"
        else
            echo "✅ Already in ~/.bashrc"
        fi
        ;;
    4)
        echo ""
        echo "✅ Binary available at: target/release/tvcode"
        echo "   Run with: ./target/release/tvcode"
        ;;
    *)
        echo "Invalid choice"
        exit 1
        ;;
esac

echo ""
echo "🎉 Installation complete!"
