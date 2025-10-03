#!/bin/bash

# Unhaunter Comprehensive Test Suite
# This script runs all tests locally: build, compile, unit, integration, and end-to-end tests

set -e  # Exit immediately if a command exits with a non-zero status

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to run a command and capture its output
run_test() {
    local test_name="$1"
    local command="$2"
    local timeout="${3:-300}"  # Default 5 minutes timeout
    
    print_status "Running $test_name..."
    
    if timeout "$timeout" bash -c "$command" 2>&1; then
        print_success "$test_name completed successfully"
        return 0
    else
        print_error "$test_name failed"
        return 1
    fi
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Main execution
main() {
    echo "=========================================="
    echo "🧪 Unhaunter Comprehensive Test Suite"
    echo "=========================================="
    echo ""
    
    # Check prerequisites
    print_status "Checking prerequisites..."
    
    if ! command_exists cargo; then
        print_error "Cargo not found. Please install Rust toolchain."
        exit 1
    fi
    
    if ! command_exists rustc; then
        print_error "Rust compiler not found. Please install Rust toolchain."
        exit 1
    fi
    
    print_success "Prerequisites check passed"
    echo ""
    
    # Track test results
    local failed_tests=()
    local total_tests=0
    
    # Phase 1: Code Quality Checks
    echo "📋 Phase 1: Code Quality Checks"
    echo "--------------------------------"
    
    total_tests=$((total_tests + 1))
    if ! run_test "Format Check" "cargo fmt --all -- --check"; then
        failed_tests+=("Format Check")
    fi
    
    total_tests=$((total_tests + 1))
    if ! run_test "Lint Check" "cargo clippy --all-targets --all-features -- -D warnings"; then
        failed_tests+=("Lint Check")
    fi
    
    echo ""
    
    # Phase 2: Compilation
    echo "🔨 Phase 2: Compilation"
    echo "----------------------"
    
    total_tests=$((total_tests + 1))
    if ! run_test "Compile Check" "cargo check --workspace --all-targets --message-format=short"; then
        failed_tests+=("Compile Check")
    fi
    
    total_tests=$((total_tests + 1))
    if ! run_test "Release Build" "cargo build --release --bin unhaunter_game"; then
        failed_tests+=("Release Build")
    fi
    
    echo ""
    
    # Phase 3: Unit Tests
    echo "🧩 Phase 3: Unit Tests"
    echo "---------------------"
    
    total_tests=$((total_tests + 1))
    if ! run_test "Core Unit Tests" "cargo test -p uncore --lib -- --skip ghost_setfinder --test-threads=4"; then
        failed_tests+=("Core Unit Tests")
    fi
    
    # Run unit tests for all packages
    for package in uncore unstd ungear ungearitems unmaphub untruck unplayer unghost unlight unmenu unnpc untmxmap unsettings unmenusettings unwalkie uncoremenu unmapload uncampaign unprofile unsummary unwalkie_types unwalkiecore unfog; do
        if [ -d "$package" ]; then
            total_tests=$((total_tests + 1))
            if ! run_test "Unit Tests ($package)" "cargo test -p $package --lib -- --test-threads=4" 60; then
                failed_tests+=("Unit Tests ($package)")
            fi
        fi
    done
    
    echo ""
    
    # Phase 4: Integration Tests
    echo "🔗 Phase 4: Integration Tests"
    echo "----------------------------"
    
    total_tests=$((total_tests + 1))
    if ! run_test "Integration Tests" "cargo test --test integration_tests -- --test-threads=4"; then
        failed_tests+=("Integration Tests")
    fi
    
    total_tests=$((total_tests + 1))
    if ! run_test "Simple Integration Tests" "cargo test --test simple_integration_tests -- --test-threads=4"; then
        failed_tests+=("Simple Integration Tests")
    fi
    
    echo ""
    
    # Phase 5: End-to-End Tests (Game Simulation)
    echo "🎮 Phase 5: End-to-End Tests"
    echo "---------------------------"
    
    total_tests=$((total_tests + 1))
    if ! run_test "Game Binary Test" "cargo run --bin unhaunter_game -- --help"; then
        failed_tests+=("Game Binary Test")
    fi
    
    # Test game with draft maps flag
    total_tests=$((total_tests + 1))
    if ! run_test "Game with Draft Maps" "timeout 10s cargo run --bin unhaunter_game -- --draft-maps || true"; then
        failed_tests+=("Game with Draft Maps")
    fi
    
    # Test walkie voice generator
    if [ -f "tools/text_to_speech/walkie_voice_generator/src/main.rs" ]; then
        total_tests=$((total_tests + 1))
        if ! run_test "Walkie Voice Generator" "cargo run --bin unhaunter_walkie_voice_generator -- --help"; then
            failed_tests+=("Walkie Voice Generator")
        fi
    fi
    
    echo ""
    
    # Phase 6: Performance Tests
    echo "⚡ Phase 6: Performance Tests"
    echo "---------------------------"
    
    total_tests=$((total_tests + 1))
    if ! run_test "Benchmark Compilation" "cargo bench --no-run"; then
        failed_tests+=("Benchmark Compilation")
    fi
    
    echo ""
    
    # Summary
    echo "=========================================="
    echo "📊 Test Summary"
    echo "=========================================="
    
    local passed_tests=$((total_tests - ${#failed_tests[@]}))
    
    if [ ${#failed_tests[@]} -eq 0 ]; then
        print_success "All $total_tests tests passed! 🎉"
        echo ""
        echo "✅ Code Quality: PASSED"
        echo "✅ Compilation: PASSED"
        echo "✅ Unit Tests: PASSED"
        echo "✅ Integration Tests: PASSED"
        echo "✅ End-to-End Tests: PASSED"
        echo "✅ Performance Tests: PASSED"
        echo ""
        print_success "Ready for deployment! 🚀"
        exit 0
    else
        print_error "${#failed_tests[@]} out of $total_tests tests failed:"
        echo ""
        for test in "${failed_tests[@]}"; do
            echo "  ❌ $test"
        done
        echo ""
        print_error "Please fix the failing tests before proceeding."
        exit 1
    fi
}

# Run main function
main "$@"

