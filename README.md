<div align="center">

# WAS - WhatsAppServer

**Minimal WhatsApp Web automation server built in Rust (sending only)**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)](Dockerfile)

[Features](#features) • [Quick Start](#quick-start) • [API](#api-reference) • [Configuration](#configuration)

</div>

---

## Features

| Feature | Description |
|---------|-------------|
| **REST API** | Send messages (text + file attachments) with OpenAPI/Swagger docs |
| **Multi-instance** | Manage multiple WhatsApp accounts simultaneously |
| **Dual Auth** | QR code scanning and phone number pairing |
| **Local Auth** | Optional JWT-based authentication for multi-user setups |

## Quick Start

### Prerequisites

- **Rust** 1.70+
- **Chrome/Chromium** browser installed
- macOS, Linux, or Windows

### Installation

```bash
# Clone
git clone https://github.com/devstroop/was.git
cd was

# Configure
cp config/app.example.toml config/app.toml

# Build & Run
cargo run --release
```

Server starts at **http://localhost:3000**

### Docker

```bash
docker-compose up -d
```

## API Reference

### Instance Management

```bash
# Create instance
curl -X POST http://localhost:3000/api/v1/instances \
  -H "Authorization: Bearer your-api-token" \
  -H "Content-Type: application/json" \
  -d '{"name": "my-whatsapp"}'

# List instances
curl http://localhost:3000/api/v1/instances \
  -H "Authorization: Bearer your-api-token"
```

### Authentication

```bash
# Get QR code (PNG image)
GET /api/v1/instances/{id}/link/qr

# Link with phone number
POST /api/v1/instances/{id}/link/phone

# Check status
GET /api/v1/instances/{id}/status

# Unlink
DELETE /api/v1/instances/{id}/unlink
```

### Messaging

```bash
# Send text message
curl -X POST "http://localhost:3000/api/v1/instances/{id}/send?phone=+1234567890&text=Hello" \
  -H "Authorization: Bearer your-api-token"

# Send file with caption
curl -X POST "http://localhost:3000/api/v1/instances/{id}/send?phone=+1234567890&text=Caption" \
  -H "Authorization: Bearer your-api-token" \
  -F "file=@image.jpg"
```

### Health

```bash
GET /api/health      # Health check
GET /api/ready       # Readiness probe (K8s)
GET /api/live        # Liveness probe (K8s)
GET /api/metrics     # Service metrics
```

**Swagger UI**: http://localhost:3000/api-docs/

## Configuration

Copy `config/app.example.toml` to `config/app.toml`:

```toml
[server]
host = "0.0.0.0"
port = 3000

[browser]
headless = true
timeout_ms = 30000

[auth]
enabled = true
secret = "your-secure-secret"

[swagger]
enabled = true
path = "/api-docs"
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `WAS_HOST` | Server host | `0.0.0.0` |
| `WAS_PORT` | Server port | `3000` |
| `WAS__AUTH__SECRET_KEY` | Secret key | - |
| `RUST_LOG` | Log level | `info` |

## Build Options

```bash
cargo build --release
```

## Project Structure

```
src/
├── bin/was.rs           # Entry point
├── handlers/api/        # REST API handlers
│   ├── instances.rs     # Instance management
│   ├── whatsapp.rs      # WhatsApp operations
│   ├── chat.rs          # Messaging (send)
│   ├── health.rs        # Health & metrics
│   ├── auth.rs          # Authentication
│   └── users.rs         # User management
├── services/            # Business logic
│   ├── whatsapp/        # WhatsApp service
│   ├── auth/            # Authentication
│   └── database/        # SQLite operations
├── browser/             # Chromium automation
├── models/              # Data structures
├── middleware/           # HTTP middleware
├── config.rs            # Configuration
└── error.rs             # Error types
```

## License

MIT License - see [LICENSE](LICENSE)

---

<div align="center">

**Built with ❤️ by [Devstroop Technologies](https://devstroop.com)**

</div>
