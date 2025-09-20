@echo off
REM Unhaunter Cross-Platform Build Script for Windows
REM Usage: build.bat [platform] [action]
REM Platforms: windows, android, all
REM Actions: build, run, release, test

setlocal enabledelayedexpansion

REM Default values
set PLATFORM=%1
set ACTION=%2

if "%PLATFORM%"=="" set PLATFORM=windows
if "%ACTION%"=="" set ACTION=build

REM Helper functions
:log_info
echo [INFO] %~1
goto :eof

:log_success
echo [SUCCESS] %~1
goto :eof

:log_warning
echo [WARNING] %~1
goto :eof

:log_error
echo [ERROR] %~1
goto :eof

REM Check if required tools are installed
:check_tools
call :log_info "Checking required tools..."

where cargo >nul 2>nul
if %errorlevel% neq 0 (
    call :log_error "Cargo is not installed. Please install Rust first."
    exit /b 1
)

where rustup >nul 2>nul
if %errorlevel% neq 0 (
    call :log_error "Rustup is not installed. Please install Rust first."
    exit /b 1
)

call :log_success "Rust toolchain is available"
goto :eof

REM Build for Windows PC (native)
:build_windows
call :log_info "Building for Windows PC..."

if "%ACTION%"=="build" (
    cargo build --bin unhaunter_game
    call :log_success "Windows build completed"
) else if "%ACTION%"=="run" (
    cargo run --bin unhaunter_game
) else if "%ACTION%"=="release" (
    cargo build --release --bin unhaunter_game
    call :log_success "Windows release build completed"
) else if "%ACTION%"=="test" (
    cargo test
    call :log_success "Windows tests completed"
) else (
    call :log_error "Unknown action: %ACTION%"
    exit /b 1
)
goto :eof

REM Build for Android
:build_android
call :log_info "Building for Android..."

if "%ANDROID_HOME%"=="" (
    call :log_warning "ANDROID_HOME is not set. Please set it to your Android SDK path."
    call :log_info "Example: set ANDROID_HOME=C:\Android\Sdk"
    exit /b 1
)

if not exist "%ANDROID_HOME%\ndk" (
    call :log_warning "Android NDK not found. Please install NDK through Android Studio SDK Manager."
    exit /b 1
)

if "%ACTION%"=="build" (
    cargo apk build --bin unhaunter_game
    call :log_success "Android build completed"
) else if "%ACTION%"=="run" (
    cargo apk run --bin unhaunter_game
) else if "%ACTION%"=="release" (
    cargo apk build --release --bin unhaunter_game
    call :log_success "Android release build completed"
) else if "%ACTION%"=="test" (
    call :log_warning "Android testing not implemented in this script"
) else (
    call :log_error "Unknown action: %ACTION%"
    exit /b 1
)
goto :eof

REM Build for all platforms
:build_all
call :log_info "Building for all platforms..."

REM Always build Windows first (native platform)
call :build_windows

REM Build Android if possible
if not "%ANDROID_HOME%"=="" (
    call :build_android
) else (
    call :log_warning "Skipping Android build - ANDROID_HOME not set"
)

call :log_success "Cross-platform build completed"
goto :eof

REM Main execution
:main
call :log_info "Starting Unhaunter cross-platform build..."
call :log_info "Platform: %PLATFORM%, Action: %ACTION%"

call :check_tools
if %errorlevel% neq 0 exit /b %errorlevel%

if "%PLATFORM%"=="windows" (
    call :build_windows
) else if "%PLATFORM%"=="android" (
    call :build_android
) else if "%PLATFORM%"=="all" (
    call :build_all
) else (
    call :log_error "Unknown platform: %PLATFORM%"
    call :log_info "Available platforms: windows, android, all"
    exit /b 1
)

call :log_success "Build process completed successfully!"
goto :eof

REM Show usage if no arguments provided
if "%1"=="" (
    echo Unhaunter Cross-Platform Build Script for Windows
    echo.
    echo Usage: %0 [platform] [action]
    echo.
    echo Platforms:
    echo   windows  - Build for Windows PC
    echo   android  - Build for Android
    echo   all      - Build for all platforms
    echo.
    echo Actions:
    echo   build    - Build the project (default)
    echo   run      - Build and run the project
    echo   release  - Build release version
    echo   test     - Run tests
    echo.
    echo Examples:
    echo   %0 windows build
    echo   %0 android run
    echo   %0 all build
    exit /b 0
)

call :main
