# CI/CD Workflows

This directory contains the streamlined CI/CD workflows for the Unhaunter project. The pipeline has been simplified into three distinct workflows, each serving a specific purpose in the development lifecycle.

## Workflow Overview

### 1. Development Workflow (`development.yml`)
**Purpose**: Latest/Features/Fixes
**Triggers**: 
- Push to feature branches (`feat/*`, `fix/*`, `cont/*`, `perf/*`)
- Pull requests to `main`

**Tasks**:
- Setup job environment
- Checkout code
- Update APT
- Install Linux Dependencies
- Setup Rust with rustfmt and clippy
- Install Just
- Build and Test (format check, lint check, compile check, unit tests)

**Duration**: ~15 minutes

### 2. Staging Workflow (`staging.yml`)
**Purpose**: Stable/Testing/QA
**Triggers**: 
- Push to `staging` branch

**Tasks**:
- Setup job environment
- Checkout code
- Update APT
- Install Linux Dependencies
- Install Windows Cross-Compile Dependencies (mingw)
- Install iOS cross-compile dependencies
- Install Android cross-compile dependencies
- Install WASM Dependencies
- Setup Rust with rustfmt and clippy
- Install Just
- Build and Test (comprehensive testing including integration tests)
- Package Artifacts
- Upload Artifacts

**Duration**: ~30 minutes

### 3. Production Workflow (`production.yml`)
**Purpose**: Live/Release Ready
**Triggers**: 
- Push to tags matching `v*.*.*` pattern

**Tasks**:
- Setup job environment
- Checkout code (tagged commit)
- Update APT
- Install Linux Dependencies
- Install Windows Cross-Compile Dependencies (mingw)
- Install iOS cross-compile dependencies
- Install Android cross-compile dependencies
- Install WASM Dependencies
- Setup Rust with rustfmt and clippy
- Install Just
- Get version from Tag
- Extract Release Notes from CHANGELOG.md
- Build and Package Artifacts (comprehensive testing and building)
- Create GitHub Release and Upload Artifacts
- Post checkout code (tagged commit)

**Duration**: ~45 minutes

## Key Features

### Streamlined Design
- Each workflow focuses on its specific purpose
- No redundant steps across workflows
- Clear separation of concerns

### Cross-Platform Support
- Linux (primary platform)
- Windows (via mingw cross-compilation)
- Android (via cargo-apk)
- iOS (via cargo-xcode)
- WASM (via wasm-pack)

### Efficient Caching
- Cargo dependencies cached across runs
- Tool installations cached when possible
- Reduced build times through smart caching

### Automated Releases
- Automatic GitHub release creation
- Release notes extraction from CHANGELOG.md
- Artifact packaging and upload

## Usage

### Development
```bash
# Create feature branch
git checkout -b feat/new-feature

# Make changes and push
git push origin feat/new-feature

# Development workflow will automatically run
```

### Staging
```bash
# Merge to staging branch
git checkout staging
git merge feat/new-feature
git push origin staging

# Staging workflow will automatically run
```

### Production
```bash
# Create and push version tag
git tag v0.3.3
git push origin v0.3.3

# Production workflow will automatically run and create release
```

## Dependencies

### Required Tools
- Rust toolchain (stable)
- Just (command runner)
- Cross-compilation tools (mingw, cargo-apk, cargo-xcode, wasm-pack)

### System Dependencies
- Linux: build-essential, libasound2-dev, pkg-config, X11 libraries, OpenGL libraries, udev libraries
- Windows: mingw-w64 cross-compiler
- Android: Android SDK, NDK, cargo-apk
- iOS: Xcode, cargo-xcode (macOS only)
- WASM: wasm-pack

## Configuration

### Environment Variables
- `CARGO_TERM_COLOR`: always
- `RUST_BACKTRACE`: 1
- `CARGO_INCREMENTAL`: 1

### Caching Strategy
- Cargo registry and git dependencies
- Target directory
- Tool installations

## Troubleshooting

### Common Issues
1. **Build failures**: Check system dependencies installation
2. **Cross-compilation issues**: Verify target toolchain installation
3. **Release failures**: Ensure CHANGELOG.md exists and has proper format
4. **Cache issues**: Clear GitHub Actions cache if builds are inconsistent

### Debugging
- Check workflow logs in GitHub Actions
- Verify all required dependencies are installed
- Ensure proper branch naming conventions are followed

## Migration Notes

This streamlined pipeline replaces the previous complex multi-stage CI/CD system with:
- 5 separate workflow files → 3 focused workflows
- Complex dependency chains → Simple linear execution
- Redundant steps → Purpose-specific tasks
- Long build times → Optimized caching and parallel execution

The new system maintains all functionality while being more maintainable and efficient.