# Testing Guide for WhatsApp Engine Rust

This document provides a comprehensive guide to testing the WhatsApp Engine Rust project.

## Test Structure

Our test suite is organized into several categories:

### 1. Unit Tests
- **Location**: `src/` (inline with source code)
- **Purpose**: Test individual functions and modules in isolation
- **Requirements**: No external dependencies
- **Command**: `cargo test --lib`

### 2. Integration Tests
- **Location**: `tests/integration_tests.rs`
- **Purpose**: Test interaction between multiple components
- **Requirements**: May require Chrome for browser integration
- **Command**: `cargo test --test integration_tests`

### 3. Browser Tests
- **Location**: `tests/browser_tests.rs`
- **Purpose**: Test browser automation functionality
- **Requirements**: Chrome/Chromium installation (for some tests)
- **Command**: `cargo test --test browser_tests`

### 4. Service Tests
- **Location**: `tests/service_tests.rs`
- **Purpose**: Test service layer functionality
- **Requirements**: Minimal dependencies
- **Command**: `cargo test --test service_tests`

## Test Categories

### Basic Tests (No Chrome Required)
These tests run without requiring Chrome installation:
- `test_browser_service_creation` - Tests service instantiation
- `test_browser_service_graceful_failure` - Tests error handling
- `test_multiple_browser_services` - Tests multiple service instances
- `test_browser_service_double_close` - Tests cleanup robustness
- `test_browser_service_config_validation` - Tests configuration handling
- `test_concurrent_browser_operations` - Tests concurrency
- `test_browser_service_memory_cleanup` - Tests memory management
- `test_browser_service_error_recovery` - Tests error recovery
- `test_browser_service_configuration_edge_cases` - Tests config edge cases

### Chrome-Dependent Tests (Require Chrome)
These tests require Chrome/Chromium to be installed:
- `test_browser_initialization` - Tests browser startup
- `test_page_creation` - Tests page creation and navigation
- `test_whatsapp_page_persistence` - Tests WhatsApp page caching
- `test_page_navigation` - Tests navigation to different URLs
- `test_browser_service_stress_test` - Tests multiple page creation
- `test_browser_service_timeout_handling` - Tests timeout scenarios
- `test_browser_service_whatsapp_specific` - Tests WhatsApp-specific logic

## Running Tests

### Using Cargo Directly

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only browser tests
cargo test --test browser_tests

# Run Chrome-dependent tests
cargo test --test browser_tests -- --ignored

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_browser_service_creation
```

### Using the Test Runner Script

We provide a comprehensive test runner script for easier test management:

```bash
# Make sure the script is executable
chmod +x scripts/test_runner.sh

# Run different test categories
./scripts/test_runner.sh unit           # Unit tests only
./scripts/test_runner.sh browser        # Browser tests
./scripts/test_runner.sh integration    # Integration tests
./scripts/test_runner.sh all           # All tests
./scripts/test_runner.sh quick         # Quick smoke tests
./scripts/test_runner.sh stress        # Stress tests (slow)
./scripts/test_runner.sh coverage      # Tests with coverage
./scripts/test_runner.sh clean         # Clean and rebuild
./scripts/test_runner.sh help          # Show help
```

## Test Configuration

### Chrome/Chromium Requirements

Some tests require Chrome or Chromium to be installed. The test runner will automatically detect:
- Google Chrome (`google-chrome`)
- Chromium (`chromium` or `chromium-browser`)
- Chrome on macOS (`/Applications/Google Chrome.app`)

If Chrome is not found, Chrome-dependent tests will be skipped with appropriate warnings.

### Environment Variables

You can customize test behavior with environment variables:

```bash
# Set log level for tests
RUST_LOG=debug cargo test

# Set test timeout
CARGO_TEST_TIMEOUT=300 cargo test

# Skip Chrome installation check
SKIP_CHROME_CHECK=1 cargo test
```

### Test Data and Fixtures

- Test configurations are generated dynamically in tests
- No external test data files are required
- Browser user data is created in temporary directories and cleaned up

## Debugging Tests

### Verbose Output
```bash
# Show test output
cargo test -- --nocapture

# Show debug logs
RUST_LOG=debug cargo test -- --nocapture

# Run single test with full output
cargo test test_browser_service_creation -- --nocapture
```

### Chrome Debugging

For Chrome-dependent tests, you can:

1. **Enable Chrome DevTools**: Modify browser config to run in non-headless mode
2. **Check Chrome logs**: Tests will output Chrome startup information
3. **Inspect user data**: Temporary Chrome profiles are logged during test runs

### Common Issues

#### Chrome Not Found
```
[WARNING] Chrome not found, skipping integration tests
```
**Solution**: Install Chrome or Chromium:
- macOS: `brew install --cask google-chrome`
- Ubuntu: `sudo apt-get install google-chrome-stable`
- Arch: `sudo pacman -S google-chrome`

#### Port Conflicts
```
Error: Failed to launch browser: Port already in use
```
**Solution**: 
- Kill existing Chrome processes: `pkill -f chrome`
- Tests use random ports to avoid conflicts

#### Permission Issues
```
Error: Failed to create user data directory
```
**Solution**: Ensure `/tmp` is writable or set `TMPDIR` environment variable

## Continuous Integration

### GitHub Actions

Tests are configured to run on multiple platforms:
- Ubuntu (with Chrome)
- macOS (with Chrome)
- Windows (Chrome-dependent tests skipped)

### Local CI Simulation

```bash
# Simulate CI environment
./scripts/test_runner.sh all

# Run coverage like CI
./scripts/test_runner.sh coverage
```

## Performance Testing

### Stress Tests
```bash
# Run comprehensive stress tests
./scripts/test_runner.sh stress

# Monitor resource usage during tests
top -p $(pgrep -f "cargo test")
```

### Memory Leak Detection
```bash
# Run with memory debugging (requires valgrind)
cargo test --test browser_tests -- --test-threads=1
```

## Test Development Guidelines

### Writing New Tests

1. **Categorize appropriately**: Unit, integration, or browser test
2. **Use proper attributes**:
   - `#[tokio::test]` for async tests
   - `#[ignore]` for tests requiring Chrome
3. **Handle errors gracefully**: Use `Result<()>` return type
4. **Clean up resources**: Always call `browser_service.close().await?`
5. **Add documentation**: Explain what the test validates

### Test Naming Conventions

- `test_[component]_[scenario]` - e.g., `test_browser_service_creation`
- `test_[feature]_[edge_case]` - e.g., `test_whatsapp_page_persistence`
- Descriptive names that explain the test purpose

### Best Practices

1. **Independent tests**: Each test should be able to run in isolation
2. **Deterministic**: Tests should produce consistent results
3. **Fast feedback**: Keep basic tests fast, mark slow tests with `#[ignore]`
4. **Clear assertions**: Use descriptive assertion messages
5. **Resource cleanup**: Always clean up browser instances and temp files

## Coverage Reports

Generate HTML coverage reports:

```bash
# Install cargo-tarpaulin if not already installed
cargo install cargo-tarpaulin

# Generate coverage report
./scripts/test_runner.sh coverage

# Open report
open coverage/tarpaulin-report.html
```

## Troubleshooting

### Test Hangs
- Check for missing `.await` on async operations
- Verify browser cleanup in test teardown
- Use timeouts for long-running operations

### Flaky Tests
- Add retry logic for network-dependent operations
- Increase timeouts for slow operations
- Use more specific element selectors

### Resource Leaks
- Monitor Chrome processes after tests
- Check for unclosed browser instances
- Verify temporary directory cleanup

For more specific issues, check the project's issue tracker or run tests with `RUST_LOG=debug` for detailed output.
