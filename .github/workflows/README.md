# CI/CD Workflows Overview

This directory contains specialized GitHub Actions workflows designed for different stages of development and deployment.

## Workflow Types

### 🚀 **Dev Workflow** (`dev.yml`)
**Purpose**: Fast feedback for development commits
**Triggers**: 
- Push to: `main`, `develop`, `feature/*`, `bugfix/*`, `hotfix/*`
- Pull requests to: `main`, `develop`

**Features**:
- Quick change detection
- Fast compilation check (5 min timeout)
- Unit tests only (10 min timeout)
- Skip integration tests for speed
- Total runtime: ~15 minutes

### 🧪 **Staging Workflow** (`staging.yml`)
**Purpose**: Thorough testing for QA and staging branches
**Triggers**:
- Push to: `staging`, `qa`, `platform-release`
- Pull requests to: `staging`, `qa`, `platform-release`

**Features**:
- Full test suite (unit + integration)
- Desktop builds (Linux, Windows, macOS)
- Mobile builds (Android, iOS)
- Artifact uploads for testing
- Total runtime: ~60-90 minutes

### 🎯 **Release Workflow** (`release.yml`)
**Purpose**: Full production builds and releases
**Triggers**:
- Release events (published)
- Push to: `main`
- Tags: `v*`

**Features**:
- Complete test suite
- All platform builds
- Release packaging
- Asset uploads to GitHub releases
- Total runtime: ~90-120 minutes

### 🖥️ **Platform-Specific Workflows**
Individual workflows for testing specific platforms:

- **`platform-linux.yml`** - Linux builds and tests
- **`platform-windows.yml`** - Windows builds and tests  
- **`platform-macos.yml`** - macOS builds and tests
- **`platform-android.yml`** - Android builds and tests
- **`platform-ios.yml`** - iOS builds and tests

**Features**:
- Manual trigger with `workflow_dispatch`
- Optional test-only mode
- Platform-specific optimizations
- Individual artifact uploads

### 🧪 **Test-Only Workflow** (`test-only.yml`)
**Purpose**: Quick validation without builds
**Triggers**:
- Manual trigger
- Push to: `main`, `develop` (test-related files only)

**Features**:
- Quick tests (15 min timeout)
- Comprehensive tests (30 min timeout)
- No artifact generation
- Focus on code quality

## Workflow Selection Guide

### For Development:
- **Quick feedback**: Use `dev.yml` (automatic on feature branches)
- **Platform testing**: Use individual platform workflows
- **Test validation**: Use `test-only.yml`

### For QA/Staging:
- **Full testing**: Use `staging.yml` (automatic on staging branches)
- **Specific platform**: Use individual platform workflows

### For Production:
- **Release builds**: Use `release.yml` (automatic on releases)
- **Main branch**: Use `release.yml` (automatic on main)

## Performance Characteristics

| Workflow | Runtime | Tests | Builds | Platforms |
|----------|---------|-------|--------|-----------|
| Dev | ~15 min | Unit only | None | None |
| Staging | ~60-90 min | Full | All | All |
| Release | ~90-120 min | Full | All | All |
| Platform | ~20-60 min | Full | Single | Single |
| Test-Only | ~15-30 min | Full | Test only | None |

## Key Features

- **Smart Caching**: Comprehensive Cargo dependency caching
- **Conditional Execution**: Only run when relevant files change
- **Parallel Testing**: 4-thread test execution
- **Incremental Builds**: Faster compilation with incremental mode
- **Error Handling**: Proper failure detection and reporting
- **Artifact Management**: Organized artifact naming and storage

## Usage Examples

### Manual Platform Testing:
```bash
# Test Linux build
gh workflow run platform-linux.yml

# Test Android build only
gh workflow run platform-android.yml

# Run tests only (no build)
gh workflow run platform-linux.yml -f test_mode=true
```

### Branch Strategy:
- **Feature branches**: Trigger `dev.yml` automatically
- **Staging branches**: Trigger `staging.yml` automatically  
- **Main branch**: Trigger `release.yml` automatically
- **Releases**: Trigger `release.yml` automatically

This setup provides maximum flexibility while maintaining efficiency and thoroughness for each development stage.
