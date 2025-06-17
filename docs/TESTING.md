# Testing Guide - WhatsApp Engine Rust 🧪

This document outlines the testing strategy, best practices, and procedures for WhatsApp Engine Rust.

## 📋 Table of Contents

- [Testing Philosophy](#-testing-philosophy)
- [Test Categories](#-test-categories)
- [Test Setup](#-test-setup)
- [Running Tests](#-running-tests)
- [Writing Tests](#-writing-tests)
- [Mock Framework](#-mock-framework)
- [Integration Testing](#-integration-testing)
- [Performance Testing](#-performance-testing)
- [CI/CD Testing](#-cicd-testing)
- [Test Coverage](#-test-coverage)
- [Debugging Tests](#-debugging-tests)
- [Best Practices](#-best-practices)

## 🎯 Testing Philosophy

### Core Principles

- **Reliability First**: Tests should be reliable and deterministic
- **Fast Feedback**: Unit tests run quickly (<100ms each)
- **Clear Intent**: Test names describe what they verify
- **Isolated**: Tests don't depend on each other
- **Maintainable**: Tests are easy to update when code changes

### Testing Pyramid

```
    /\     E2E Tests (Few)
   /  \    - Full workflow testing
  /____\   - Real browser automation
 /      \  Integration Tests (Some)
/        \ - Service interaction testing
\        / - Real components with mocks
 \______/  Unit Tests (Many)
           - Fast, isolated, comprehensive
           - Mock external dependencies
```

## 🧪 Test Categories

### 1. Unit Tests

**Purpose**: Test individual functions, methods, and components in isolation.

**Characteristics**:
- Fast execution (<100ms each)
- No external dependencies
- Mock or stub external services
- High coverage of edge cases

**Location**: Inline with source code using `#[cfg(test)]`

```rust
// src/services/auth_service.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_phone_number_validation() {
        let service = AuthService::new_mock();
        
        // Valid phone number
        let result = service.validate_phone("+1234567890").await;
        assert!(result.is_ok());
        
        // Invalid phone number
        let result = service.validate_phone("invalid").await;
        assert!(result.is_err());
    }
}
```

### 2. Integration Tests

**Purpose**: Test interaction between services and components.

**Characteristics**:
- Test real service interactions
- Use test configurations
- May use real browser instances
- Slower than unit tests but still reasonably fast

**Location**: `tests/` directory

```rust
// tests/integration_tests.rs
use whatsapp_engine::{WhatsAppEngine, AppConfig};

#[tokio::test]
async fn test_engine_authentication_flow() {
    let config = test_config();
    let engine = WhatsAppEngine::new(config).await.unwrap();
    
    // Test authentication state
    let is_auth = engine.is_authenticated().await.unwrap();
    assert!(!is_auth); // Should start unauthenticated
    
    engine.close().await.unwrap();
}
```

### 3. End-to-End Tests

**Purpose**: Test complete user workflows from start to finish.

**Characteristics**:
- Use real WhatsApp Web interface
- Test actual browser automation
- Slowest but most comprehensive
- May require special setup (authenticated sessions)

**Location**: `tests/e2e/` directory

```rust
// tests/e2e/message_flow_test.rs
#[tokio::test]
#[ignore] // Run only when specifically requested
async fn test_complete_message_sending_flow() {
    let engine = create_authenticated_engine().await.unwrap();
    
    let result = engine.send_message(TEST_PHONE, "Hello from tests!").await;
    assert!(result.is_ok());
    assert!(result.unwrap().success);
    
    engine.close().await.unwrap();
}
```

## ⚙️ Test Setup

### Dependencies

Add to `Cargo.toml`:

```toml
[dev-dependencies]
tokio-test = "0.4"
mockall = "0.12"
tempfile = "3.8"
assert_matches = "1.5"
serial_test = "3.0"
criterion = "0.5"  # For benchmarks

# Test utilities
env_logger = "0.10"
tracing-test = "0.2"
```

### Test Configuration

Create `tests/common/mod.rs`:

```rust
use whatsapp_engine::{AppConfig, BrowserConfig, LoggingConfig};
use std::sync::Once;

static INIT: Once = Once::new();

pub fn init_test_logging() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

pub fn test_config() -> AppConfig {
    AppConfig {
        browser: BrowserConfig {
            headless: true,
            timeout_ms: 10000,
            args: vec![
                "--no-sandbox".to_string(),
                "--disable-dev-shm-usage".to_string(),
                "--disable-gpu".to_string(),
                "--remote-debugging-port=9223".to_string(), // Different port for tests
            ],
        },
        logging: LoggingConfig {
            level: "debug".to_string(),
            file: None, // Don't write log files during tests
        },
        ..Default::default()
    }
}

pub async fn create_test_engine() -> Result<WhatsAppEngine> {
    init_test_logging();
    WhatsAppEngine::new(test_config()).await
}

pub fn test_phone_number() -> &'static str {
    std::env::var("TEST_PHONE_NUMBER")
        .unwrap_or_else(|_| "+1234567890".to_string())
        .leak()
}
```

### Environment Setup

Create `.env.test`:

```env
# Test-specific environment variables
RUST_LOG=debug
BROWSER_HEADLESS=true
BROWSER_TIMEOUT_MS=10000
TEST_PHONE_NUMBER=+1234567890
TEST_MODE=true
```

## 🏃 Running Tests

### Basic Commands

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_phone_authentication

# Run tests in specific file
cargo test --test integration_tests

# Run only unit tests
cargo test --lib

# Run with multiple threads
cargo test -- --test-threads=4

# Run ignored tests (like E2E)
cargo test -- --ignored
```

### Test Categories

```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# End-to-end tests
cargo test --test '*' -- --ignored

# Performance tests
cargo test perf_ -- --ignored

# Browser tests specifically
cargo test browser --test '*'
```

### Parallel vs Sequential

Some tests need to run sequentially (browser tests):

```rust
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_browser_auth() {
    // This test runs sequentially with other #[serial] tests
}
```

### Test Filtering

```bash
# Run tests matching pattern
cargo test auth

# Exclude tests matching pattern
cargo test -- --skip browser

# Run tests in package
cargo test -p whatsapp-engine

# Run with specific features
cargo test --features api-server
```

## ✍️ Writing Tests

### Test Structure

Use the **Arrange-Act-Assert** pattern:

```rust
#[tokio::test]
async fn test_message_validation() {
    // Arrange
    let service = ChatService::new_mock();
    let invalid_message = "";
    
    // Act
    let result = service.send_message("+1234567890", invalid_message).await;
    
    // Assert
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        WhatsAppError::InvalidInput { field, .. } if field == "message"
    ));
}
```

### Test Naming

Use descriptive names that explain the scenario:

```rust
#[tokio::test]
async fn test_send_message_with_empty_phone_number_returns_validation_error() {
    // Test implementation
}

#[tokio::test]
async fn test_authentication_succeeds_with_valid_qr_code() {
    // Test implementation
}

#[tokio::test]
async fn test_browser_connection_retries_on_network_failure() {
    // Test implementation
}
```

### Error Testing

Test both success and failure cases:

```rust
#[tokio::test]
async fn test_authentication_errors() {
    let service = AuthService::new_mock();
    
    // Test network error
    service.mock_network_error();
    let result = service.authenticate_qr().await;
    assert!(matches!(result.unwrap_err(), WhatsAppError::Network(_)));
    
    // Test timeout
    service.mock_timeout();
    let result = service.authenticate_qr().await;
    assert!(matches!(result.unwrap_err(), WhatsAppError::Timeout { .. }));
}
```

### Async Testing

Use `tokio::test` for async tests:

```rust
#[tokio::test]
async fn test_async_operation() {
    let service = create_service().await;
    let result = service.async_method().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_timeout_handling() {
    let service = create_service().await;
    
    let result = tokio::time::timeout(
        Duration::from_millis(100),
        service.slow_operation()
    ).await;
    
    assert!(result.is_err()); // Should timeout
}
```

## 🎭 Mock Framework

### Using Mockall

```rust
use mockall::{automock, predicate::*};

#[automock]
pub trait BrowserService {
    async fn navigate(&self, url: &str) -> Result<()>;
    async fn is_connected(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_with_mock_browser() {
        let mut mock_browser = MockBrowserService::new();
        
        mock_browser
            .expect_navigate()
            .with(eq("https://web.whatsapp.com"))
            .times(1)
            .returning(|_| Ok(()));
        
        mock_browser
            .expect_is_connected()
            .returning(|| true);
        
        let service = ChatService::new(Arc::new(mock_browser));
        let result = service.initialize().await;
        assert!(result.is_ok());
    }
}
```

### Manual Mocks

```rust
pub struct MockAuthService {
    should_fail: bool,
    auth_result: Option<bool>,
}

impl MockAuthService {
    pub fn new() -> Self {
        Self {
            should_fail: false,
            auth_result: None,
        }
    }
    
    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
    
    pub fn with_auth_result(mut self, result: bool) -> Self {
        self.auth_result = Some(result);
        self
    }
}

#[async_trait]
impl AuthService for MockAuthService {
    async fn is_authenticated(&self) -> Result<bool> {
        if self.should_fail {
            return Err(WhatsAppError::Network("Mock failure".to_string()));
        }
        Ok(self.auth_result.unwrap_or(false))
    }
}
```

## 🔗 Integration Testing

### Service Integration

Test how services work together:

```rust
// tests/service_integration_test.rs
use whatsapp_engine::services::{AuthService, ChatService, BrowserService};

#[tokio::test]
async fn test_auth_chat_integration() {
    let browser = BrowserService::new_test().await.unwrap();
    let auth = AuthService::new(browser.clone());
    let chat = ChatService::new(browser.clone(), auth.clone());
    
    // Test that chat service respects auth state
    let result = chat.send_message("+1234567890", "test").await;
    assert!(matches!(result.unwrap_err(), WhatsAppError::Authentication(_)));
    
    // Mock authentication
    auth.mock_authenticated();
    
    // Now chat should work
    let result = chat.send_message("+1234567890", "test").await;
    assert!(result.is_ok());
}
```

### Database Integration

If using databases:

```rust
use tempfile::tempdir;
use whatsapp_engine::storage::SessionStore;

#[tokio::test]
async fn test_session_persistence() {
    let temp_dir = tempdir().unwrap();
    let store = SessionStore::new(temp_dir.path()).await.unwrap();
    
    // Test session saving
    let session_data = SessionData::new("test_session");
    store.save_session(&session_data).await.unwrap();
    
    // Test session loading
    let loaded = store.load_session("test_session").await.unwrap();
    assert_eq!(loaded.id, session_data.id);
}
```

## 🚀 Performance Testing

### Benchmark Tests

Create `benches/` directory for benchmarks:

```rust
// benches/message_sending.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use whatsapp_engine::WhatsAppEngine;

async fn send_message_benchmark() {
    let engine = WhatsAppEngine::new_mock().await.unwrap();
    let _ = engine.send_message("+1234567890", "Benchmark message").await;
}

fn criterion_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("send_message", |b| {
        b.to_async(&rt).iter(|| async {
            send_message_benchmark().await
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
```

Run benchmarks:

```bash
cargo bench
```

### Load Testing

```rust
// tests/load_test.rs
use futures::future::join_all;

#[tokio::test]
#[ignore]
async fn test_concurrent_message_sending() {
    let engine = create_authenticated_engine().await.unwrap();
    
    let mut tasks = Vec::new();
    for i in 0..100 {
        let engine_clone = engine.clone();
        let task = tokio::spawn(async move {
            engine_clone.send_message(
                TEST_PHONE,
                &format!("Load test message {}", i)
            ).await
        });
        tasks.push(task);
    }
    
    let results = join_all(tasks).await;
    let successful = results.iter().filter(|r| r.is_ok()).count();
    
    assert!(successful > 90, "Expected >90% success rate, got {}/100", successful);
}
```

## 🔄 CI/CD Testing

### GitHub Actions Configuration

`.github/workflows/test.yml`:

```yaml
name: Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      # Add any required services (Redis, etc.)
      
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
        components: rustfmt, clippy
    
    - name: Install Chrome
      run: |
        wget -q -O - https://dl.google.com/linux/linux_signing_key.pub | sudo apt-key add -
        echo "deb [arch=amd64] http://dl.google.com/linux/chrome/deb/ stable main" | sudo tee /etc/apt/sources.list.d/google-chrome.list
        sudo apt-get update
        sudo apt-get install -y google-chrome-stable
    
    - name: Cache cargo registry
      uses: actions/cache@v3
      with:
        path: ~/.cargo/registry
        key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Run format check
      run: cargo fmt --all -- --check
    
    - name: Run clippy
      run: cargo clippy --all-targets --all-features -- -D warnings
    
    - name: Run unit tests
      run: cargo test --lib
    
    - name: Run integration tests
      run: cargo test --test '*'
      env:
        RUST_LOG: debug
        BROWSER_HEADLESS: true
    
    - name: Run E2E tests
      run: cargo test --test '*' -- --ignored
      if: github.event_name == 'push' && github.ref == 'refs/heads/main'
      env:
        TEST_PHONE_NUMBER: ${{ secrets.TEST_PHONE_NUMBER }}
```

### Test Data Management

Use environment variables for test configuration:

```rust
use std::env;

pub struct TestConfig {
    pub phone_number: String,
    pub timeout_ms: u64,
    pub headless: bool,
}

impl TestConfig {
    pub fn from_env() -> Self {
        Self {
            phone_number: env::var("TEST_PHONE_NUMBER")
                .unwrap_or_else(|_| "+1234567890".to_string()),
            timeout_ms: env::var("TEST_TIMEOUT_MS")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .unwrap_or(10000),
            headless: env::var("CI").is_ok() || 
                      env::var("BROWSER_HEADLESS").unwrap_or_default() == "true",
        }
    }
}
```

## 📊 Test Coverage

### Generating Coverage Reports

Install `cargo-tarpaulin`:

```bash
cargo install cargo-tarpaulin
```

Generate coverage:

```bash
# Basic coverage
cargo tarpaulin

# HTML report
cargo tarpaulin --out html

# Multiple formats
cargo tarpaulin --out xml --out html

# Exclude integration tests
cargo tarpaulin --skip-clean --ignore-tests
```

### Coverage Goals

- **Unit Tests**: >90% line coverage
- **Integration Tests**: >80% feature coverage
- **Critical Paths**: 100% coverage (auth, message sending)

### Coverage Configuration

`.tarpaulin.toml`:

```toml
[tool.tarpaulin]
skip-clean = true
run-types = ["Tests", "Doctests"]
exclude-files = [
    "src/bin/*",
    "tests/*",
    "examples/*"
]
ignore-panics = true
```

## 🐛 Debugging Tests

### Debug Output

```rust
#[tokio::test]
async fn test_with_debug_output() {
    env_logger::try_init().ok(); // Initialize logging
    
    let engine = create_test_engine().await.unwrap();
    
    // Add debug logging
    tracing::debug!("Starting test");
    
    let result = engine.send_message(TEST_PHONE, "test").await;
    
    tracing::debug!("Result: {:?}", result);
    
    assert!(result.is_ok());
}
```

### Test Isolation

Use temporary directories and cleanup:

```rust
use tempfile::tempdir;

#[tokio::test]
async fn test_with_cleanup() {
    let temp_dir = tempdir().unwrap();
    let config = AppConfig {
        storage_path: temp_dir.path().to_path_buf(),
        ..test_config()
    };
    
    let engine = WhatsAppEngine::new(config).await.unwrap();
    
    // Test logic here
    
    engine.close().await.unwrap();
    // temp_dir is automatically cleaned up
}
```

### Browser Debugging

For debugging browser automation:

```rust
#[tokio::test]
async fn test_browser_debug() {
    let config = AppConfig {
        browser: BrowserConfig {
            headless: false, // Show browser window
            timeout_ms: 60000, // Longer timeout
            args: vec![
                "--remote-debugging-port=9222".to_string(),
                "--no-first-run".to_string(),
            ],
        },
        ..test_config()
    };
    
    let engine = WhatsAppEngine::new(config).await.unwrap();
    
    // Add breakpoint here to inspect browser
    std::thread::sleep(std::time::Duration::from_secs(30));
    
    engine.close().await.unwrap();
}
```

## 📝 Best Practices

### 1. Test Organization

```rust
// Group related tests in modules
mod authentication_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_qr_auth() { /* ... */ }
    
    #[tokio::test]
    async fn test_phone_auth() { /* ... */ }
}

mod messaging_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_send_text() { /* ... */ }
    
    #[tokio::test]
    async fn test_send_file() { /* ... */ }
}
```

### 2. Shared Test Utilities

```rust
// tests/common/assertions.rs
pub fn assert_phone_valid(phone: &str) {
    assert!(phone.starts_with('+'));
    assert!(phone.len() >= 8);
    assert!(phone.len() <= 15);
}

pub fn assert_message_sent(result: &SendMessageResult) {
    assert!(result.success);
    assert!(result.message_id.is_some());
    assert!(result.error.is_none());
}
```

### 3. Test Data Builders

```rust
pub struct ContactBuilder {
    contact: Contact,
}

impl ContactBuilder {
    pub fn new() -> Self {
        Self {
            contact: Contact {
                id: "test_id".to_string(),
                name: "Test Contact".to_string(),
                phone: "+1234567890".to_string(),
                is_business: false,
                profile_picture_url: None,
            },
        }
    }
    
    pub fn with_name(mut self, name: &str) -> Self {
        self.contact.name = name.to_string();
        self
    }
    
    pub fn with_phone(mut self, phone: &str) -> Self {
        self.contact.phone = phone.to_string();
        self
    }
    
    pub fn build(self) -> Contact {
        self.contact
    }
}

#[tokio::test]
async fn test_with_builder() {
    let contact = ContactBuilder::new()
        .with_name("John Doe")
        .with_phone("+1987654321")
        .build();
    
    // Use contact in test
}
```

### 4. Error Scenarios

Test all error paths:

```rust
#[tokio::test]
async fn test_all_auth_errors() {
    let service = AuthService::new_mock();
    
    // Network errors
    service.mock_network_error();
    let result = service.authenticate().await;
    assert!(matches!(result.unwrap_err(), WhatsAppError::Network(_)));
    
    // Timeout errors
    service.mock_timeout();
    let result = service.authenticate().await;
    assert!(matches!(result.unwrap_err(), WhatsAppError::Timeout { .. }));
    
    // Browser errors
    service.mock_browser_error();
    let result = service.authenticate().await;
    assert!(matches!(result.unwrap_err(), WhatsAppError::BrowserConnection(_)));
}
```

### 5. Performance Considerations

```rust
#[tokio::test]
async fn test_performance_requirements() {
    let engine = create_test_engine().await.unwrap();
    
    let start = std::time::Instant::now();
    let result = engine.send_message(TEST_PHONE, "test").await;
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    assert!(duration < std::time::Duration::from_secs(5), 
           "Message sending took too long: {:?}", duration);
}
```

This comprehensive testing guide ensures high-quality, reliable code with good test coverage and maintainable test suites.
