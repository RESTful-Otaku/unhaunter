#!/bin/bash

# Unhaunter Cross-Platform Build Script
# Usage: ./build.sh [platform] [action]
# Platforms: windows, android, ios, all
# Actions: build, run, release, test

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
PLATFORM=${1:-"all"}
ACTION=${2:-"build"}

# Helper functions
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

# Check if required tools are installed
check_tools() {
    log_info "Checking required tools..."
    
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo is not installed. Please install Rust first."
        exit 1
    fi
    
    if ! command -v rustup &> /dev/null; then
        log_error "Rustup is not installed. Please install Rust first."
        exit 1
    fi
    
    log_success "Rust toolchain is available"
}

# Build for Windows PC (native)
build_windows() {
    log_info "Building for Windows PC..."
    
    case $ACTION in
        "build")
            cargo build --bin unhaunter_game
            log_success "Windows build completed"
            ;;
        "run")
            cargo run --bin unhaunter_game
            ;;
        "release")
            cargo build --release --bin unhaunter_game
            log_success "Windows release build completed"
            ;;
        "test")
            cargo test
            log_success "Windows tests completed"
            ;;
        *)
            log_error "Unknown action: $ACTION"
            exit 1
            ;;
    esac
}

# Build for Android
build_android() {
    log_info "Building for Android..."
    
    # Check if Android SDK is configured
    if [ -z "$ANDROID_HOME" ]; then
        log_warning "ANDROID_HOME is not set. Please set it to your Android SDK path."
        log_info "Example: export ANDROID_HOME=/path/to/android/sdk"
        exit 1
    fi
    
    # Check if NDK is available
    if [ ! -d "$ANDROID_HOME/ndk" ]; then
        log_warning "Android NDK not found. Please install NDK through Android Studio SDK Manager."
        exit 1
    fi
    
    case $ACTION in
        "build")
            cargo apk build --bin unhaunter_game
            log_success "Android build completed"
            ;;
        "run")
            cargo apk run --bin unhaunter_game
            ;;
        "release")
            cargo apk build --release --bin unhaunter_game
            log_success "Android release build completed"
            ;;
        "test")
            log_warning "Android testing not implemented in this script"
            ;;
        *)
            log_error "Unknown action: $ACTION"
            exit 1
            ;;
    esac
}

# Build for iOS
build_ios() {
    log_info "Building for iOS..."
    
    # Check if we're on macOS
    if [[ "$OSTYPE" != "darwin"* ]]; then
        log_error "iOS builds require macOS with Xcode installed"
        exit 1
    fi
    
    # Check if Xcode is installed
    if ! command -v xcodebuild &> /dev/null; then
        log_error "Xcode is not installed. Please install Xcode from the App Store."
        exit 1
    fi
    
    case $ACTION in
        "build")
            cargo xcode build --bin unhaunter_game
            log_success "iOS Xcode project generated"
            ;;
        "run")
            cargo xcode run --bin unhaunter_game
            ;;
        "release")
            cargo xcode build --release --bin unhaunter_game
            log_success "iOS release Xcode project generated"
            ;;
        "test")
            log_warning "iOS testing requires opening the Xcode project"
            ;;
        *)
            log_error "Unknown action: $ACTION"
            exit 1
            ;;
    esac
}

# Build for all platforms
build_all() {
    log_info "Building for all platforms..."
    
    # Always build Windows first (native platform)
    build_windows
    
    # Build Android if possible
    if [ ! -z "$ANDROID_HOME" ]; then
        build_android
    else
        log_warning "Skipping Android build - ANDROID_HOME not set"
    fi
    
    # Build iOS if on macOS
    if [[ "$OSTYPE" == "darwin"* ]]; then
        build_ios
    else
        log_warning "Skipping iOS build - not on macOS"
    fi
    
    log_success "Cross-platform build completed"
}

# Main execution
main() {
    log_info "Starting Unhaunter cross-platform build..."
    log_info "Platform: $PLATFORM, Action: $ACTION"
    
    check_tools
    
    case $PLATFORM in
        "windows")
            build_windows
            ;;
        "android")
            build_android
            ;;
        "ios")
            build_ios
            ;;
        "all")
            build_all
            ;;
        *)
            log_error "Unknown platform: $PLATFORM"
            log_info "Available platforms: windows, android, ios, all"
            exit 1
            ;;
    esac
    
    log_success "Build process completed successfully!"
}

# Show usage if no arguments provided
if [ $# -eq 0 ]; then
    echo "Unhaunter Cross-Platform Build Script"
    echo ""
    echo "Usage: $0 [platform] [action]"
    echo ""
    echo "Platforms:"
    echo "  windows  - Build for Windows PC"
    echo "  android  - Build for Android"
    echo "  ios      - Build for iOS (macOS only)"
    echo "  all      - Build for all platforms"
    echo ""
    echo "Actions:"
    echo "  build    - Build the project (default)"
    echo "  run      - Build and run the project"
    echo "  release  - Build release version"
    echo "  test     - Run tests"
    echo ""
    echo "Examples:"
    echo "  $0 windows build"
    echo "  $0 android run"
    echo "  $0 ios release"
    echo "  $0 all build"
    exit 0
fi

main
