#!/bin/bash

# Unhaunter Platform Setup Script
# This script helps set up the development environment for cross-platform builds

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Rust is installed
check_rust() {
    log_info "Checking Rust installation..."
    
    if ! command -v rustc &> /dev/null; then
        log_error "Rust is not installed. Please install Rust first:"
        log_info "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    log_success "Rust is installed: $(rustc --version)"
}

# Install Rust targets for cross-compilation
install_rust_targets() {
    log_info "Installing Rust targets for cross-compilation..."
    
    # Android targets
    rustup target add aarch64-linux-android
    rustup target add armv7-linux-androideabi
    
    # iOS targets (only on macOS)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        rustup target add aarch64-apple-ios
        rustup target add x86_64-apple-ios
        log_success "iOS targets installed"
    else
        log_warning "Skipping iOS targets - not on macOS"
    fi
    
    log_success "Rust targets installed"
}

# Install cargo tools
install_cargo_tools() {
    log_info "Installing cargo tools..."
    
    # Install cargo-apk for Android builds
    cargo install cargo-apk
    
    # Install cargo-xcode for iOS builds (macOS only)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        cargo install cargo-xcode
        log_success "cargo-xcode installed"
    else
        log_warning "Skipping cargo-xcode - not on macOS"
    fi
    
    log_success "Cargo tools installed"
}

# Check Android setup
check_android_setup() {
    log_info "Checking Android setup..."
    
    if [ -z "$ANDROID_HOME" ]; then
        log_warning "ANDROID_HOME is not set"
        log_info "To set up Android development:"
        log_info "1. Install Android Studio"
        log_info "2. Install Android SDK and NDK through SDK Manager"
        log_info "3. Set ANDROID_HOME environment variable:"
        log_info "   export ANDROID_HOME=/path/to/android/sdk"
        log_info "   export PATH=\$PATH:\$ANDROID_HOME/tools:\$ANDROID_HOME/platform-tools"
        return 1
    fi
    
    if [ ! -d "$ANDROID_HOME/ndk" ]; then
        log_warning "Android NDK not found at $ANDROID_HOME/ndk"
        log_info "Please install NDK through Android Studio SDK Manager"
        return 1
    fi
    
    log_success "Android setup looks good"
    return 0
}

# Check iOS setup (macOS only)
check_ios_setup() {
    if [[ "$OSTYPE" != "darwin"* ]]; then
        log_warning "iOS development requires macOS"
        return 1
    fi
    
    log_info "Checking iOS setup..."
    
    if ! command -v xcodebuild &> /dev/null; then
        log_error "Xcode is not installed"
        log_info "Please install Xcode from the App Store"
        return 1
    fi
    
    # Check if Xcode license is accepted
    if ! xcodebuild -checkFirstLaunchStatus &> /dev/null; then
        log_warning "Xcode license not accepted"
        log_info "Run: sudo xcodebuild -license accept"
        return 1
    fi
    
    log_success "iOS setup looks good"
    return 0
}

# Main setup function
main() {
    log_info "Setting up Unhaunter cross-platform development environment..."
    
    check_rust
    install_rust_targets
    install_cargo_tools
    
    log_info "Checking platform-specific setups..."
    
    android_ok=false
    ios_ok=false
    
    if check_android_setup; then
        android_ok=true
    fi
    
    if check_ios_setup; then
        ios_ok=true
    fi
    
    log_info "Setup summary:"
    log_info "- Rust toolchain: ✓"
    log_info "- Cross-compilation targets: ✓"
    log_info "- Cargo tools: ✓"
    log_info "- Android setup: $([ "$android_ok" = true ] && echo "✓" || echo "⚠️  (see warnings above)")"
    log_info "- iOS setup: $([ "$ios_ok" = true ] && echo "✓" || echo "⚠️  (see warnings above)")"
    
    log_info ""
    log_info "You can now use the following commands:"
    log_info "- ./build.sh windows build    # Windows build"
    if [ "$android_ok" = true ]; then
        log_info "- ./build.sh android build   # Android build"
    fi
    if [ "$ios_ok" = true ]; then
        log_info "- ./build.sh ios build       # iOS build"
    fi
    log_info "- ./build.sh all build        # All platforms"
    log_info ""
    log_info "Or use the Justfile commands:"
    log_info "- just build-windows"
    log_info "- just build-android"
    log_info "- just build-ios"
    log_info "- just build-all"
    
    log_success "Setup completed!"
}

main
