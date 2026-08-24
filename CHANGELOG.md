# Changelog

All notable changes to WhatsApp Engine Rust will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.1] - 2026-08-24

### Fixed
- Minimal configs (e.g. just `[server]` + `[auth]`) no longer fail boot with `missing field allow_methods` — every top-level section/field now carries serde defaults matching `AppConfig::default()` (#56)
- `data/attachments/.staging` runtime working dir is now properly ignored via `.gitignore` (stray file removed)

## [0.5.0] - 2026-08-24

Web admin UI release: single-binary server-rendered dashboard (htmx 4 + uikit), messaging hardening, and pre-production security posture.

### Added
- **Web admin UI at `/app`** (#27–#33): login with inline errors, live dashboard (5s polling), instances management (CRUD, QR-link flow polling every 2s until linked, config editor), messaging console (text + multipart media through the hardened send path), users & access-tokens admin (one-time secret reveal)
  - htmx 4.0.0-beta6 + uikit htmx dist + theme tokens vendored and embedded via rust-embed — **no Node toolchain**, release stays a single artifact
  - Cookie sessions reusing the API token store (TTL'd, default 7 days); stateless per-session CSRF on every mutation; swap-friendly error fragments throughout
- **Opt-in static secret key** (#39): no default key ships anywhere; when `[auth] secret_key` is set (≥16 chars) the superadmin Bearer path works, otherwise it is disabled and user tokens are the only path
- **Admin bootstrap** (#40): the first registered user becomes Admin automatically
- **Brute-force protection** (#44): sliding-window throttle on API + web login (per IP|username) and register (per IP) → 429 + Retry-After; `[auth.rate_limits]` configurable
- **Session expiry + logout-all** (#42): `[auth] session_ttl_hours`; `POST /api/v1/auth/logout-all` and "Sign out everywhere"
- **User editing** (#45): role change, activate/deactivate (rejected mid-session), password reset revoking web sessions; self-demotion/self-deactivation guards
- **Staging janitor** (#46): hourly purge of stale uploads (`[storage] staging_ttl_hours`, default 24h); successful sends delete staged files immediately
- **Browser visibility** (#47): boot warning when Chrome/Chromium missing, `browser_available` in `/api/health`, dashboard banner

### Fixed
- Rate limiter was constructed per-request (windows reset every call) — one shared limiter now lives on `InstanceManager` (#24)
- `/api/v1/instances/:id/send` routed through `SendService` end-to-end incl. multipart media upload (#25)
- CORS honors `[cors] allow_origins` instead of hardcoded wildcard (#43)

### Removed
- MCP surface (SSE endpoint, `mcp` cargo feature, docs rows) (#23)
- Legacy `handlers/api/chat.rs` send path superseded by the thin handler

## [0.4.0] - 2026-08-22

Architecture refactor release: clean layered architecture (domain → application → infrastructure → interfaces), hardened auth, decomposed god files, and wired messaging with rate limiting.

### Added
- **Layered architecture**: `domain/` (pure entities, no axum/tokio/rusqlite), `application/` (use-cases + ports), `infrastructure/` (browser/persistence/config/security adapters), `interfaces/http/` (router, DTOs, middleware stack, thin handlers)
- **Instance registry** (`application/instance::InstanceRegistry`): pure in-memory registry (metadata + config + phone index) with typed `RegistryError`; `SqliteInstanceStore` implements the `InstanceStore` port
- **Typed config validation**: bounds-checked instance config (idle timeout, browser timeout, rate limits) with stable error codes (`invalid_idle_timeout`, …) mapped to HTTP 400; `restart_required` derived from actual browser-field diff on `PUT /api/v1/instances/:id/config`
- **Per-instance observability**: atomic counters (messages sent / errors / warmups / last activity) via `shared::observability::instance_metrics`, exposed in `/api/metrics`
- **Messaging ports wiring**: `SendService` (validator → rate limit → browser) behind `BrowserSendPort`/`RateLimitPort`; sliding-window rate limiter per instance from config (`messages_per_minute`, default 60/min) returning HTTP 429
- **OpenAPI stability boundary**: versioned DTOs under `interfaces/http/dto` with DTO↔domain mappers (`TryFrom`/`From`) and committed `openapi.snapshot.json` — domain changes no longer break the API contract without an explicit DTO change
- **Identity domain split**: `domain/identity/{user,token,permission}` value objects with validation (`validate_username`, `validate_password`, RBAC hierarchy Owner > Operator > Viewer)

### Changed
- **bin/was.rs thinned**: 418 → 116 LOC bootstrap; router + middleware stack extracted to `interfaces::http::{router::build_full_router, middleware::http_middleware_stack}` — unit-testable without browser/database
- **God files decomposed** (no file in the verticals exceeds ~400 LOC):
  - `services/whatsapp/instance.rs` 1221 → 554 LOC (lifecycle + auth watcher split into `instance_lifecycle.rs` 315 LOC / `instance_auth.rs` 390 LOC)
  - `handlers/api/users.rs` 845 LOC split into `interfaces/http/handlers/identity/{users,tokens,assignments,me}` (<150 LOC each)
  - `services/whatsapp/instance_manager.rs` now a facade over registry + store
- **Secret handling hardened**: prod boot fails on default/weak secret (`SecretValidator::validate` env-aware); constant-time Bearer compare in auth middleware; single SHA256 token-hash source
- **State machine formalized**: `SLEEPING → WARMING_UP → ACTIVE → ERROR` transitions validated by `application::instance::InstanceState` with tests
- Test suite grown 29 → 95 unit tests (state machine, registry, repos, validators, mappers, snapshot)

### Security
- Default secret rejected outside development; weak secrets (<16 chars) rejected always
- Timing-safe secret comparison prevents Bearer timing attacks
- Config errors return typed codes instead of leaking internals via generic 500

### Compatibility
- REST API contracts unchanged: `/api/health|ready|live|metrics`, `/api/v1/instances/*`, `/api/v1/users/*`, `/api/v1/auth/*`
- `restart_required` in config responses is now accurate instead of always `true`

## [0.3.0] and earlier

See git history for versions prior to the layered-architecture refactor.

## [0.2.0] - 2024-12-18

### Added
- **Library Mode**: Complete reusable Rust library with clean async API
- **Dual Operation**: Support for both library usage and standalone API server
- **Enhanced Authentication**: Simplified state machine for QR and phone authentication
- **Comprehensive API**: Full REST API with OpenAPI/Swagger documentation
- **Browser Service**: Robust browser lifecycle management with Chromium integration
- **Configuration Management**: TOML and environment variable configuration
- **Health Monitoring**: Comprehensive health checks
- **Docker Support**: Production-ready containerization with optimized builds
- **Error Handling**: Rich error types with retry guidance and context
- **Documentation**: Extensive developer documentation and quick reference guides
- **Examples**: Working examples for both library and API usage
- **Testing Framework**: Integration tests with browser automation

### Core Features
- **WhatsAppEngine**: Main library entry point with async-first design
- **AuthService**: Authentication and session management
- **ChatService**: Messaging operations and chat management
- **BrowserService**: Browser control and WhatsApp Web integration
- **Message Processing**: Text messages with file attachment support
- **Contact Management**: Contact retrieval and management
- **Session Persistence**: Automatic session state management
- **Rate Limiting**: Built-in rate limiting for API protection
- **CORS Support**: Configurable cross-origin resource sharing

### API Endpoints
- `POST /auth/qr` - Generate QR code for authentication
- `POST /auth/phone` - Authenticate using phone number
- `GET /auth/status` - Check authentication status
- `POST /auth/logout` - Logout and clear session
- `POST /messages/send` - Send text messages
- `POST /messages/send-file` - Send file attachments
- `GET /contacts` - Retrieve contacts list
- `GET /chats` - Retrieve chat list
- `GET /api/health` - Health check endpoint

### Documentation
- **README.md**: Complete project overview with architecture diagrams
- **DEVELOPER_GUIDE.md**: Comprehensive library development guide
- **LIBRARY_QUICK_REFERENCE.md**: Quick reference for library usage
- **API_REFERENCE.md**: Complete REST API documentation
- **SECURITY.md**: Security best practices and hardening guide
- **DEPLOYMENT_GUIDE.md**: Production deployment procedures
- **ARCHITECTURE_REVIEW.md**: Current state analysis and improvement roadmap

### Changed
- **Architecture**: Moved from simple script to production-ready service architecture
- **Error Handling**: Implemented comprehensive error types with context
- **Configuration**: Unified configuration system with multiple sources
- **Browser Management**: Enhanced browser lifecycle with proper cleanup
- **Performance**: Optimized for production use with resource management

### Technical Improvements
- **Async/Await**: Full async implementation using Tokio
- **Type Safety**: Strong typing with comprehensive error handling
- **Resource Management**: Proper cleanup with Drop implementations
- **Thread Safety**: Arc-wrapped services for safe sharing
- **Memory Management**: Efficient memory usage with smart pointers
- **Network Optimization**: Connection pooling and timeout handling

### Infrastructure
- **Docker**: Multi-stage builds with optimized production images
- **Kubernetes**: Deployment manifests with health checks and resource limits
- **CI/CD**: GitHub Actions with automated testing and building
- **Security**: Comprehensive security scanning and best practices

## [0.1.0] - 2024-11-15

### Added
- Initial WhatsApp Engine implementation
- Basic browser automation with Chromium
- Simple message sending functionality
- QR code authentication support
- Basic error handling
- Docker containerization
- Initial documentation

### Core Components
- **Browser Integration**: Chromium-based WhatsApp Web automation
- **Authentication**: QR code scanning for session establishment
- **Message Sending**: Basic text message transmission
- **Configuration**: Environment-based configuration
- **Logging**: Basic logging infrastructure

### Features
- WhatsApp Web automation using chromiumoxide
- QR code generation and scanning workflow
- Session persistence for authentication
- Basic REST API for message sending
- Docker support for containerized deployment
- Rust-based implementation for performance and safety

### Technical Stack
- **Language**: Rust 2021 edition
- **Browser**: Chromium via chromiumoxide
- **Async Runtime**: Tokio
- **Web Framework**: Axum (basic implementation)
- **Serialization**: Serde with JSON support
- **Configuration**: Environment variables
- **Containerization**: Docker with Debian base

### Documentation
- Basic README with setup instructions
- Docker deployment guide
- API endpoint documentation
- Example usage patterns

### Known Limitations
- Limited error handling and recovery
- Basic authentication flow only
- No session management
- Limited API functionality
- No comprehensive testing
- Minimal monitoring and observability

---

## Release Process

### Version Numbering

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Incompatible API changes
- **MINOR**: New functionality in a backwards compatible manner
- **PATCH**: Backwards compatible bug fixes

### Release Checklist

- [ ] Update version in `Cargo.toml`
- [ ] Update `CHANGELOG.md` with release notes
- [ ] Ensure all tests pass
- [ ] Update documentation
- [ ] Create GitHub release with binaries
- [ ] Publish to crates.io (when public)
- [ ] Update Docker images
- [ ] Announce release

### Breaking Changes Policy

Breaking changes will be:
1. Clearly documented in changelog
2. Deprecated in advance when possible
3. Communicated through multiple channels
4. Accompanied by migration guides

### Support Policy

- **Current major version**: Full support with bug fixes and security updates
- **Previous major version**: Security updates only for 6 months
- **Older versions**: No support (users encouraged to upgrade)

### Security Updates

Security fixes will be:
- Released as patch versions for supported versions
- Documented in changelog with CVE references when applicable
- Announced through security advisories
- Backported to previous major version when severity is high

---

## Contributing to Changelog

When contributing, please:

1. Add entries to the "Unreleased" section
2. Follow the established format and categories
3. Include relevant issue/PR references
4. Use clear, descriptive language
5. Categorize changes appropriately (Added/Changed/Deprecated/Removed/Fixed/Security)

Example entry format:
```markdown
### Added
- New feature description [#123](https://github.com/org/repo/pull/123)
- Another feature with context and benefits

### Fixed
- Bug fix description and impact [#456](https://github.com/org/repo/issues/456)
```
