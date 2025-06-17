# Contributing to WhatsApp Engine Rust 🤝

We welcome contributions to WhatsApp Engine Rust! This document provides guidelines for contributing to the project.

## 📋 Table of Contents

- [Code of Conduct](#-code-of-conduct)
- [Getting Started](#-getting-started)
- [Development Environment](#-development-environment)
- [Project Structure](#-project-structure)
- [Development Workflow](#-development-workflow)
- [Testing Guidelines](#-testing-guidelines)
- [Code Style and Standards](#-code-style-and-standards)
- [Pull Request Process](#-pull-request-process)
- [Issue Reporting](#-issue-reporting)
- [Documentation](#-documentation)
- [Security](#-security)
- [Production Readiness Assessment](docs/PRODUCTION_READINESS_ASSESSMENT.md)
- [Implementation Issues & Development Tasks](docs/IMPLEMENTATION_ISSUES.md)
- [Development Iteration Plan](docs/ITERATION_PLAN.md)
- [Architecture Review](docs/ARCHITECTURE_REVIEW.md)

## 📜 Code of Conduct

By participating in this project, you agree to abide by our code of conduct:

- **Be respectful** and inclusive to all participants
- **Be constructive** in discussions and feedback
- **Focus on what is best** for the community
- **Show empathy** towards other community members

## 🚀 Getting Started

### Prerequisites

- **Rust 1.70+** (latest stable recommended)
- **Chrome/Chromium browser** (for WhatsApp Web automation)
- **Git** for version control
- **Docker** (optional, for containerized development)

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/whatsapp-engine-rust.git
   cd whatsapp-engine-rust
   ```

3. Add the upstream remote:
   ```bash
   git remote add upstream https://github.com/devstroop/whatsapp-engine-rust.git
   ```

## 🛠️ Development Environment

### Quick Setup

```bash
# Install dependencies and build
cargo build

# Run tests
cargo test

# Start development server
cargo run --bin whatsapp-server --features api-server

# Generate documentation
cargo doc --open
```

### Development with Docker

```bash
# Build development image
docker build -f docker/Dockerfile -t whatsapp-engine-dev .

# Run with volume mount for development
docker run -v $(pwd):/app -p 3000:3000 whatsapp-engine-dev
```

### IDE Setup

**VS Code (Recommended)**
Install these extensions:
- `rust-analyzer` - Rust language support
- `CodeLLDB` - Debugging support
- `crates` - Cargo.toml helper
- `Test Explorer` - Test integration

**.vscode/settings.json**:
```json
{
    "rust-analyzer.cargo.features": ["full"],
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.procMacro.enable": true
}
```

## 🏗️ Project Structure

```
src/
├── lib.rs              # Main library entry point & WhatsAppEngine
├── error.rs            # Error types and handling
├── main.rs            # Binary crate entry point (for CLI tools)
├── auth/              # Authentication middleware
├── bin/               # Binary targets (servers, CLI tools)
├── config/            # Configuration management
├── handlers/          # HTTP request handlers (API endpoints)
├── middleware/        # Request/response middleware
├── models/            # Domain models and data structures
├── services/          # Core business logic services
├── locators/          # UI element locators for browser automation
└── utils/             # Utilities and helpers

tests/                 # Integration tests
examples/              # Usage examples
docs/                  # Documentation
config/               # Configuration files
docker/               # Docker configurations
scripts/              # Utility scripts
```

### Service Architecture

- **AuthService**: Authentication and session management
- **ChatService**: Messaging operations and chat management
- **BrowserService**: Browser lifecycle and WhatsApp Web integration
- **WhatsAppEngine**: Main orchestrator and public API

## 🔄 Development Workflow

### Branch Strategy

- **`main`** - Stable, production-ready code
- **`develop`** - Integration branch for features
- **`feature/feature-name`** - Feature development
- **`bugfix/bug-description`** - Bug fixes
- **`hotfix/critical-fix`** - Critical production fixes

### Workflow Steps

1. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following our coding standards

3. **Write tests** for new functionality

4. **Run the full test suite**:
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

5. **Update documentation** if needed

6. **Commit your changes**:
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```

7. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

8. **Create a Pull Request** on GitHub

### Commit Message Convention

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Build process or auxiliary tool changes

**Examples:**
```
feat(auth): implement phone number authentication
fix(browser): resolve connection timeout issues
docs(api): update authentication endpoint documentation
test(chat): add unit tests for message sending
```

## 🧪 Testing Guidelines

### Test Categories

**Unit Tests**
- Test individual functions and methods
- Mock external dependencies
- Fast execution (< 100ms each)
- Located in `src/` files with `#[cfg(test)]`

**Integration Tests**
- Test service interactions
- Use real browser instances
- Located in `tests/` directory
- Longer execution time acceptable

**End-to-End Tests**
- Test complete user workflows
- Use real WhatsApp Web interface
- May require authentication setup
- Run in CI with special configuration

### Writing Tests

```rust
// Unit test example
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_message_validation() {
        let result = validate_phone_number("+1234567890");
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_invalid_phone_number() {
        let result = validate_phone_number("invalid");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WhatsAppError::InvalidInput { .. }));
    }
}

// Integration test example
#[tokio::test]
async fn test_engine_creation() {
    let config = AppConfig::default();
    let engine = WhatsAppEngine::new(config).await;
    assert!(engine.is_ok());
}
```

### Test Configuration

Create `tests/common/mod.rs` for shared test utilities:

```rust
use whatsapp_engine::{AppConfig, BrowserConfig};

pub fn test_config() -> AppConfig {
    AppConfig {
        browser: BrowserConfig {
            headless: true,
            timeout_ms: 10000,
            args: vec!["--no-sandbox".to_string()],
        },
        // ... other test-specific config
    }
}

pub async fn create_test_engine() -> Result<WhatsAppEngine> {
    WhatsAppEngine::new(test_config()).await
}
```

### Test Requirements

- **Coverage**: Aim for >80% line coverage
- **Documentation**: Document complex test scenarios
- **Cleanup**: Properly clean up resources (close browsers, etc.)
- **Isolation**: Tests should not depend on each other
- **Speed**: Keep unit tests under 100ms each

### Running Tests

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration_tests

# Specific test
cargo test test_phone_authentication

# With output
cargo test -- --nocapture

# Coverage report (requires cargo-tarpaulin)
cargo tarpaulin --out html
```

## 🎨 Code Style and Standards

### Rust Style

Follow the official [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/):

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Lint code
cargo clippy

# Check with all warnings
cargo clippy -- -W clippy::all
```

### Code Quality Rules

**Naming Conventions:**
- `snake_case` for functions, variables, modules
- `PascalCase` for types, traits, enums
- `SCREAMING_SNAKE_CASE` for constants
- Descriptive names over short names

**Error Handling:**
- Use `Result<T, WhatsAppError>` for fallible operations
- Provide context in error messages
- Use `?` operator for error propagation
- Handle all error cases explicitly

**Documentation:**
- Document all public APIs with `///`
- Include examples in doc comments
- Use `#[doc = "..."]` for generated docs
- Keep documentation up-to-date

**Async Code:**
- Use `async`/`await` consistently
- Avoid blocking operations in async contexts
- Use `tokio::time::timeout` for timeouts
- Handle cancellation gracefully

### Example Good Code

```rust
use crate::{WhatsAppError, Result};

/// Validates a phone number in international format.
/// 
/// # Arguments
/// 
/// * `phone` - Phone number to validate (e.g., "+1234567890")
/// 
/// # Example
/// 
/// ```rust
/// use whatsapp_engine::validate_phone_number;
/// 
/// let result = validate_phone_number("+1234567890");
/// assert!(result.is_ok());
/// ```
/// 
/// # Errors
/// 
/// Returns `WhatsAppError::InvalidInput` if the phone number format is invalid.
pub fn validate_phone_number(phone: &str) -> Result<String> {
    if !phone.starts_with('+') {
        return Err(WhatsAppError::InvalidInput {
            field: "phone_number".to_string(),
            reason: "Must start with country code (+)".to_string(),
        });
    }
    
    if phone.len() < 8 || phone.len() > 15 {
        return Err(WhatsAppError::InvalidInput {
            field: "phone_number".to_string(),
            reason: "Must be 8-15 digits including country code".to_string(),
        });
    }
    
    Ok(phone.to_string())
}
```

## 🔍 Pull Request Process

### Before Submitting

- [ ] Code follows style guidelines (`cargo fmt` and `cargo clippy` pass)
- [ ] Tests are written and passing (`cargo test`)
- [ ] Documentation is updated
- [ ] Changes are described in commit messages
- [ ] Branch is up-to-date with `main`

### PR Template

Use this template for your pull requests:

```markdown
## 📝 Description
Brief description of the changes.

## 🎯 Type of Change
- [ ] Bug fix (non-breaking change)
- [ ] New feature (non-breaking change)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## 🧪 Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed
- [ ] All tests pass

## 📋 Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No new warnings introduced

## 🔗 Related Issues
Fixes #(issue number)
```

### Review Process

1. **Automated Checks**: All CI checks must pass
2. **Code Review**: At least one maintainer approval required
3. **Testing**: All tests must pass
4. **Documentation**: Documentation must be updated
5. **Merge**: Squash and merge to `main`

## 🐛 Issue Reporting

### Bug Reports

Use the bug report template:

```markdown
**Bug Description**
Clear description of the bug.

**Steps to Reproduce**
1. Step one
2. Step two
3. Step three

**Expected Behavior**
What should happen.

**Actual Behavior**
What actually happens.

**Environment**
- OS: [e.g., Ubuntu 22.04]
- Rust version: [e.g., 1.75.0]
- WhatsApp Engine version: [e.g., 0.2.0]
- Browser: [e.g., Chrome 120.0]

**Additional Context**
Any additional information, logs, screenshots.
```

### Feature Requests

Use the feature request template:

```markdown
**Feature Description**
Clear description of the feature.

**Use Case**
Why is this feature needed?

**Proposed Solution**
How should it work?

**Alternatives Considered**
Other solutions you've considered.

**Additional Context**
Any additional information.
```

## 📚 Documentation

### Types of Documentation

1. **Code Comments**: Inline explanations
2. **Doc Comments**: Rust documentation (`///`)
3. **README**: Project overview and quick start
4. **Guides**: Step-by-step tutorials
5. **API Reference**: Complete API documentation
6. **Architecture**: System design and patterns

### Documentation Standards

- **Clear and Concise**: Use simple language
- **Examples**: Include code examples
- **Up-to-date**: Keep in sync with code changes
- **Complete**: Cover all public APIs
- **Searchable**: Use consistent terminology

### Building Documentation

```bash
# Generate API docs
cargo doc --open

# Check doc links
cargo doc --no-deps

# Build all documentation
cargo doc --workspace --no-deps
```

## 🔒 Security

### Security Guidelines

- **No secrets in code**: Use environment variables
- **Input validation**: Validate all user inputs
- **Secure defaults**: Use secure configurations by default
- **Dependency scanning**: Regularly update dependencies
- **Code review**: All security-related code needs extra review

### Reporting Security Issues

**DO NOT** open public issues for security vulnerabilities.

Instead, email security concerns to: security@devstroop.com

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We will respond within 48 hours and provide a timeline for fixes.

## 🎯 Getting Help

### Current Development Status

⚠️ **Important**: This project has critical code-level issues that prevent basic functionality. **Start with concrete code fixes** before architecture work:

- **[🔥 Critical Code Issues](docs/CRITICAL_CODE_ISSUES.md)** - 10 immediate code fixes with exact locations
- **[Implementation Issues](docs/IMPLEMENTATION_ISSUES.md)** - Complete list of known issues and development tasks
- **[Production Readiness Assessment](docs/PRODUCTION_READINESS_ASSESSMENT.md)** - Current production readiness status
- **[Roadmap](ROADMAP.md)** - Development timeline and priorities

### 🔥 **URGENT: Focus on Code-Level Issues First**

**Start here for immediate impact:**
- **[Critical Code Issues](docs/CRITICAL_CODE_ISSUES.md)** - 10 specific code fixes with exact file locations and implementation details

### Priority Contribution Areas

**Immediate Code Fixes Needed** (Start Week 1):
1. **🔥 Session Management** - `src/lib.rs:633-634` - Hardcoded None values
2. **� File Sending** - `src/lib.rs:706` - Not implemented, returns placeholder
3. **🔥 Data Retrieval** - `src/lib.rs:728,744` - Returns fake hardcoded data
4. **🔥 Health Checks** - `src/lib.rs:757-758` - Hardcoded true, no real status

**Core Implementation Gaps** (Week 2-3):
5. **🔥 Browser Service Methods** - Missing core methods called by main engine
6. **🔥 Input Validation** - No validation on any endpoints (security risk)
7. **🔥 Graceful Shutdown** - Server doesn't handle signals, data loss risk
8. **🔥 Authentication Integration** - New phone auth not integrated

See [Critical Code Issues](docs/CRITICAL_CODE_ISSUES.md) for exact fixes needed.

### Resources

- **Documentation**: [docs/](docs/)
- **Examples**: [examples/](examples/)
- **Discussions**: GitHub Discussions
- **Issues**: GitHub Issues (for bugs and features)

### Community

- **GitHub Discussions**: General questions and discussions
- **Discord**: Real-time chat (link in README)
- **Email**: info@devstroop.com for business inquiries

### Mentorship

New contributors welcome! Look for issues labeled:
- `good first issue` - Perfect for new contributors
- `help wanted` - Community help needed
- `mentor available` - Guidance provided

---

Thank you for contributing to WhatsApp Engine Rust! Your contributions help make this project better for everyone. 🚀
