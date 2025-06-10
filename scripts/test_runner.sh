#!/bin/bash

# Test Runner Script for WhatsApp Engine Rust
# This script provides different test execution modes for comprehensive testing

set -e

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

# Function to check if Chrome is available
check_chrome() {
    if command -v google-chrome >/dev/null 2>&1; then
        return 0
    elif command -v chromium >/dev/null 2>&1; then
        return 0
    elif command -v chromium-browser >/dev/null 2>&1; then
        return 0
    elif [ -d "/Applications/Google Chrome.app" ]; then
        return 0
    else
        return 1
    fi
}

# Help function
show_help() {
    echo "WhatsApp Engine Rust Test Runner"
    echo ""
    echo "Usage: $0 [OPTION]"
    echo ""
    echo "Options:"
    echo "  unit           Run unit tests only (no Chrome required)"
    echo "  integration    Run integration tests (requires Chrome)"
    echo "  browser        Run browser tests (requires Chrome)"
    echo "  all            Run all tests"
    echo "  stress         Run stress tests (requires Chrome, slow)"
    echo "  quick          Run quick smoke tests"
    echo "  coverage       Run tests with coverage report"
    echo "  clean          Clean test artifacts and rebuild"
    echo "  help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 unit          # Run only unit tests"
    echo "  $0 browser       # Run browser tests with Chrome"
    echo "  $0 all           # Run complete test suite"
    echo ""
}

# Function to run unit tests
run_unit_tests() {
    print_status "Running unit tests..."
    cargo test --lib
    print_success "Unit tests completed"
}

# Function to run integration tests
run_integration_tests() {
    print_status "Running integration tests..."
    if check_chrome; then
        cargo test --test integration_tests
        print_success "Integration tests completed"
    else
        print_warning "Chrome not found, skipping integration tests"
        return 1
    fi
}

# Function to run browser tests
run_browser_tests() {
    print_status "Running browser tests..."
    if check_chrome; then
        # Run basic browser tests
        cargo test --test browser_tests -- --skip ignore
        print_success "Basic browser tests completed"
        
        # Ask if user wants to run ignored tests (require Chrome)
        echo ""
        read -p "Run Chrome-dependent tests? These require Chrome to be installed (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            print_status "Running Chrome-dependent browser tests..."
            cargo test --test browser_tests -- --ignored
            print_success "Chrome-dependent browser tests completed"
        fi
    else
        print_warning "Chrome not found, running only basic browser tests"
        cargo test --test browser_tests -- --skip ignore
    fi
}

# Function to run stress tests
run_stress_tests() {
    print_status "Running stress tests..."
    if check_chrome; then
        print_warning "Stress tests may take several minutes to complete..."
        cargo test --test browser_tests test_browser_service_stress_test -- --ignored --nocapture
        print_success "Stress tests completed"
    else
        print_error "Chrome required for stress tests"
        return 1
    fi
}

# Function to run quick smoke tests
run_quick_tests() {
    print_status "Running quick smoke tests..."
    cargo test test_browser_service_creation
    cargo test test_browser_service_graceful_failure
    print_success "Quick tests completed"
}

# Function to run all tests
run_all_tests() {
    print_status "Running complete test suite..."
    
    # Unit tests
    run_unit_tests
    
    # Service tests
    print_status "Running service tests..."
    cargo test --test service_tests
    
    # Browser tests (basic)
    cargo test --test browser_tests -- --skip ignore
    
    # Integration tests if Chrome available
    if check_chrome; then
        run_integration_tests
        
        echo ""
        read -p "Run Chrome-dependent tests? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            cargo test --test browser_tests -- --ignored
        fi
    fi
    
    print_success "All tests completed"
}

# Function to run tests with coverage
run_coverage_tests() {
    print_status "Running tests with coverage..."
    
    # Check if cargo-tarpaulin is installed
    if ! command -v cargo-tarpaulin >/dev/null 2>&1; then
        print_warning "cargo-tarpaulin not found, installing..."
        cargo install cargo-tarpaulin
    fi
    
    cargo tarpaulin --out Html --output-dir coverage --skip-clean
    print_success "Coverage report generated in coverage/tarpaulin-report.html"
}

# Function to clean and rebuild
clean_and_rebuild() {
    print_status "Cleaning test artifacts..."
    cargo clean
    print_status "Rebuilding project..."
    cargo build
    print_success "Clean and rebuild completed"
}

# Main script logic
case "${1:-help}" in
    unit)
        run_unit_tests
        ;;
    integration)
        run_integration_tests
        ;;
    browser)
        run_browser_tests
        ;;
    all)
        run_all_tests
        ;;
    stress)
        run_stress_tests
        ;;
    quick)
        run_quick_tests
        ;;
    coverage)
        run_coverage_tests
        ;;
    clean)
        clean_and_rebuild
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        print_error "Unknown option: $1"
        echo ""
        show_help
        exit 1
        ;;
esac
