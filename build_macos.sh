#!/bin/bash

# Build script for macOS Flipper Monitor app
# This script builds the Rust binary and packages it into a .app bundle

set -e

APP_NAME="Flipper Monitor"
BUNDLE_ID="com.yourcompany.flipper-monitor"
BINARY_NAME="flipper-monitor-macos"
VERSION="1.0"

echo "Building Flipper Monitor for macOS..."

# Build the release binary
echo "Step 1: Building Rust binary..."
cargo build --release

# Create app bundle structure
echo "Step 2: Creating app bundle structure..."
APP_DIR="target/release/${APP_NAME}.app"
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

# Copy binary
echo "Step 3: Copying binary..."
cp "target/release/${BINARY_NAME}" "$APP_DIR/Contents/MacOS/${BINARY_NAME}"
chmod +x "$APP_DIR/Contents/MacOS/${BINARY_NAME}"

# Copy Info.plist
echo "Step 4: Copying Info.plist..."
cp Info.plist "$APP_DIR/Contents/Info.plist"

# Optional: Copy icon if you have one
# cp icon.icns "$APP_DIR/Contents/Resources/icon.icns"

echo "Step 5: Setting permissions..."
chmod -R 755 "$APP_DIR"

# Optional: Code signing (requires Apple Developer account)
# echo "Step 6: Code signing..."
# codesign --force --deep --sign "Developer ID Application: Your Name" "$APP_DIR"

echo ""
echo "✅ Build complete!"
echo "App bundle created at: $APP_DIR"
echo ""
echo "To run the app:"
echo "  open \"$APP_DIR\""
echo ""
echo "To create a DMG for distribution:"
echo "  hdiutil create -volname \"${APP_NAME}\" -srcfolder \"$APP_DIR\" -ov -format UDZO \"${APP_NAME}.dmg\""
