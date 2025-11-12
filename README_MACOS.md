# Flipper Monitor - macOS Version

This guide covers building and running the Flipper Monitor app on macOS.

## Prerequisites

1. **Rust toolchain** (install via [rustup](https://rustup.rs/))
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Xcode Command Line Tools**
   ```bash
   xcode-select --install
   ```

3. **macOS 13.0+** (Ventura or later recommended)

## Project Structure

```
flipper-monitor-macos/
├── Cargo.toml              # Rust dependencies and build config
├── Info.plist              # macOS app metadata and permissions
├── build_macos.sh          # Build script for creating .app bundle
├── src/
│   ├── main.rs             # Entry point
│   ├── flipper_manager.rs  # BLE communication with Flipper
│   ├── helpers.rs          # Utility functions
│   ├── system_info.rs      # System monitoring (with macOS GPU support)
│   └── gpu_info_macos.rs   # macOS-specific GPU information
```

## Building the App

### Quick Build (Binary Only)

```bash
cargo build --release
```

The binary will be at `target/release/flipper-monitor-macos`

### Full Build (App Bundle)

```bash
chmod +x build_macos.sh
./build_macos.sh
```

This creates a complete `.app` bundle at `target/release/Flipper Monitor.app`

## Running the App

### From Terminal

```bash
cargo run --release
```

### From App Bundle

```bash
open "target/release/Flipper Monitor.app"
```

## macOS-Specific Features

### Bluetooth Permissions

The app requires Bluetooth access to communicate with Flipper Zero. The first time you run the app, macOS will prompt you to grant Bluetooth permission. This is configured in `Info.plist`:

```xml
<key>NSBluetoothAlwaysUsageDescription</key>
<string>This app needs Bluetooth to communicate with your Flipper Zero device</string>
```

### GPU Monitoring

The macOS version includes platform-specific GPU monitoring:

#### **Apple Silicon (M1/M2/M3)**
- Uses `ioreg` to query GPU memory
- Uses `sysctl` to get unified memory info
- Attempts `powermetrics` for GPU usage (requires elevated privileges)

#### **Intel/AMD GPUs**
- Uses `system_profiler SPDisplaysDataType` for VRAM info
- Limited real-time usage monitoring (macOS doesn't expose this easily)

### Supported Methods

1. **`ioreg`** - IORegistry query for GPU hardware info
2. **`system_profiler`** - System information for displays/GPUs
3. **`sysctl`** - System control for CPU and memory info
4. **`powermetrics`** - Power and performance metrics (may require sudo)

## Known Limitations

### GPU Usage Monitoring

Real-time GPU usage percentage is **limited on macOS**:

- **Apple Silicon**: `powermetrics` can provide GPU active residency, but requires `sudo` access
- **Intel/AMD**: No reliable real-time usage API available without private frameworks
- **Workaround**: The app attempts multiple methods and falls back to 0% if unavailable

### VRAM on Apple Silicon

Apple Silicon uses **unified memory architecture**:
- GPU shares system RAM
- VRAM values represent total system memory
- Actual GPU memory usage is managed dynamically by the OS

## Distribution

### Create DMG for Distribution

```bash
hdiutil create -volname "Flipper Monitor" \
  -srcfolder "target/release/Flipper Monitor.app" \
  -ov -format UDZO \
  "FlipperMonitor.dmg"
```

### Code Signing (Optional)

For distribution outside the App Store, you'll need to sign the app:

```bash
codesign --force --deep --sign "Developer ID Application: Your Name" \
  "target/release/Flipper Monitor.app"
```

### Notarization (Optional)

For Gatekeeper approval:

```bash
xcrun notarytool submit FlipperMonitor.dmg \
  --apple-id "your@email.com" \
  --team-id "TEAMID" \
  --password "app-specific-password"
```

## Troubleshooting

### Bluetooth Not Working

1. Check System Settings → Privacy & Security → Bluetooth
2. Ensure the app has permission
3. Try resetting Bluetooth module:
   ```bash
   sudo pkill bluetoothd
   ```

### GPU Info Returns Zero

This is normal on macOS due to limited API access. Options:

1. Run with `sudo` to enable `powermetrics` (not recommended for production)
2. Accept limited GPU monitoring
3. Use Activity Monitor as reference

### Build Errors

If you get linking errors:

```bash
# Clean and rebuild
cargo clean
cargo build --release
```

If you get framework errors:

```bash
# Ensure Xcode Command Line Tools are installed
xcode-select --install
```

## Development

### Testing GPU Info

```bash
# Test GPU detection
cargo test --release

# Run with verbose logging
RUST_LOG=debug cargo run --release
```

### Debugging BLE

```bash
# List Bluetooth devices
system_profiler SPBluetoothDataType

# Monitor Bluetooth activity
log stream --predicate 'subsystem == "com.apple.bluetooth"'
```

## Dependencies

- **btleplug**: Cross-platform Bluetooth LE library (uses CoreBluetooth on macOS)
- **sysinfo**: System information library (macOS support via native APIs)
- **tokio**: Async runtime
- **metal** (macOS only): Metal framework bindings for GPU access
- **io-kit-sys** (macOS only): IOKit framework for hardware queries
- **core-foundation** (macOS only): Core Foundation framework

## Platform Compatibility

| Feature | macOS Intel | macOS Apple Silicon | Notes |
|---------|-------------|---------------------|-------|
| CPU Monitoring | ✅ | ✅ | Full support |
| RAM Monitoring | ✅ | ✅ | Full support |
| BLE Communication | ✅ | ✅ | Full support |
| GPU Detection | ✅ | ✅ | Full support |
| VRAM Detection | ✅ | ⚠️ | Unified memory on Apple Silicon |
| GPU Usage % | ⚠️ | ⚠️ | Limited without sudo |

## Next Steps

1. **Add GUI**: Consider using [egui](https://github.com/emilk/egui) or [tauri](https://tauri.app/) for a native UI
2. **Menu Bar App**: Convert to a menu bar utility using [tray-item](https://github.com/olback/tray-item-rs)
3. **Notifications**: Add system notifications for alerts
4. **Auto-start**: Add launch agent for automatic startup
5. **Settings**: Add preferences for update intervals, units, etc.

## License

[Your License Here]

## Support

For issues specific to macOS, please check:
- Bluetooth permissions in System Settings
- Console.app for crash logs
- Activity Monitor for resource usage
