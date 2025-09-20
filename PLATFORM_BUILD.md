# Cross-Platform Build Guide

This document explains how to build Unhaunter for Windows PC, Android, and iOS platforms.

## Prerequisites

### All Platforms
- Rust toolchain (latest stable)
- Git

### Windows PC
- No additional requirements (builds natively)

### Android
- Android SDK with NDK installed
- Set `ANDROID_HOME` environment variable to your Android SDK path
- Example: `export ANDROID_HOME=/path/to/android/sdk`

### iOS (macOS only)
- macOS with Xcode installed
- iOS Simulator (optional, for testing)

## Quick Start

### Using the Build Scripts

#### Linux/macOS
```bash
# Build for all platforms
./build.sh all build

# Build for specific platform
./build.sh windows build
./build.sh android run
./build.sh ios release
```

#### Windows
```cmd
REM Build for all platforms
build.bat all build

REM Build for specific platform
build.bat windows build
build.bat android run
```

### Manual Build Commands

#### Windows PC
```bash
# Debug build
cargo build --bin unhaunter_game

# Release build
cargo build --release --bin unhaunter_game

# Run
cargo run --bin unhaunter_game

# Test
cargo test
```

#### Android
```bash
# Debug build
cargo apk build --bin unhaunter_game

# Release build
cargo apk build --release --bin unhaunter_game

# Run on device/emulator
cargo apk run --bin unhaunter_game
```

#### iOS
```bash
# Generate Xcode project
cargo xcode build --bin unhaunter_game

# Generate release Xcode project
cargo xcode build --release --bin unhaunter_game

# Run on simulator
cargo xcode run --bin unhaunter_game
```

## Platform-Specific Configuration

### Android Configuration
The Android configuration is defined in `Cargo.toml` under `[package.metadata.android]`:

```toml
[package.metadata.android]
app_name = "Unhaunter"
package_name = "com.unhaunter.game"
version_name = "0.3.2"
version_code = 302
min_sdk_version = 21
target_sdk_version = 34
compile_sdk_version = 34
permissions = [
    "android.permission.INTERNET",
    "android.permission.WRITE_EXTERNAL_STORAGE",
    "android.permission.READ_EXTERNAL_STORAGE",
    "android.permission.RECORD_AUDIO",
]
features = ["android-native-activity"]
```

### iOS Configuration
The iOS configuration is defined in `Cargo.toml` under `[package.metadata.ios]`:

```toml
[package.metadata.ios]
app_name = "Unhaunter"
bundle_identifier = "com.unhaunter.game"
version = "0.3.2"
build_number = 302
minimum_deployment_target = "12.0"
features = ["metal"]
```

## Development Workflow

### 1. Local Development
- Use `cargo run --bin unhaunter_game` for quick testing on your native platform
- Use `cargo test` to run all tests

### 2. Cross-Platform Testing
- Use the build scripts to test on different platforms
- Android: Ensure device/emulator is connected and `adb devices` shows your device
- iOS: Use Xcode Simulator or connect a physical device

### 3. Release Builds
- Use `--release` flag for optimized builds
- GitHub Actions automatically builds all platforms on release

## Troubleshooting

### Android Issues

#### ANDROID_HOME not set
```bash
export ANDROID_HOME=/path/to/android/sdk
export PATH=$PATH:$ANDROID_HOME/tools:$ANDROID_HOME/platform-tools
```

#### NDK not found
- Install NDK through Android Studio SDK Manager
- Ensure NDK is in `$ANDROID_HOME/ndk/`

#### Device not recognized
```bash
adb devices
adb kill-server
adb start-server
```

### iOS Issues

#### Xcode not found
- Install Xcode from App Store
- Accept Xcode license: `sudo xcodebuild -license accept`

#### iOS targets not installed
```bash
rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios
```

### General Issues

#### Rust toolchain issues
```bash
rustup update
rustup component add rustfmt clippy
```

#### Clean builds
```bash
cargo clean
cargo build
```

## CI/CD Integration

The project includes GitHub Actions workflows that automatically:
- Test on Windows, macOS, and Linux
- Build for all platforms on push/PR
- Create release artifacts on release

See `.github/workflows/cross-platform.yml` for details.

## Performance Optimization

### Release Profile
The project uses an optimized release profile in `Cargo.toml`:
- Size optimization (`opt-level = "s"`)
- Thin LTO enabled
- Debug info stripped
- Single codegen unit for better optimization

### Platform-Specific Optimizations
- **Android**: Uses `android-native-activity` feature
- **iOS**: Uses `metal` renderer for better performance
- **Windows**: Native DirectX/Vulkan support through Bevy

## Asset Management

All platform builds share the same asset directory (`assets/`). Bevy automatically handles platform-specific asset loading and optimization.

## Version Management

Update version information in:
1. `Cargo.toml` workspace version
2. `[package.metadata.android]` version_name and version_code
3. `[package.metadata.ios]` version and build_number

## Support

For platform-specific issues:
- Check Bevy documentation: https://bevyengine.org/
- Android: https://bevyengine.org/learn/book/getting-started/setup/#android
- iOS: https://bevyengine.org/learn/book/getting-started/setup/#ios
