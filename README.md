<div align="center">

# WAS - WhatsApp Server

**High-performance WhatsApp Web automation server built in Rust**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)](Dockerfile)

[Features](#features) • [Quick Start](#quick-start) • [API](#api-reference) • [MCP](#mcp-model-context-protocol) • [Configuration](#configuration)

</div>

---

## Features

| Feature | Description |
|---------|-------------|
| **Web Dashboard** | Built-in HTMX UI with WhatsApp-style design - no build step |
| **REST API** | Full messaging API with OpenAPI/Swagger documentation |
| **MCP Server** | Model Context Protocol for AI agent integration (Claude, etc.) |
| **Dual Auth** | QR code scanning and phone number pairing |
| **Real-time** | SSE event streams for live message updates |
| **Webhooks** | Push notifications with HMAC signature verification |
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
cargo run --release --features mcp
```

Server starts at **http://localhost:3000**

### Docker

```bash
docker-compose up -d
```

## Web Dashboard

The web UI is served automatically at `/` - no separate build required.

| Page | Path | Description |
|------|------|-------------|
| Dashboard | `/` | Server health, connection status, quick actions |
| Authentication | `/auth` | QR code & phone pairing |
| Chats | `/chats` | WhatsApp-style chat interface |
| Webhooks | `/webhooks` | Configure webhook endpoints |
| Tokens | `/tokens` | Manage API access tokens |
| Settings | `/settings` | Theme, session management |

## API Reference

### Authentication

```bash
# Get auth status
GET /api/v1/auth/status

# Get QR code (base64 PNG)
GET /api/v1/auth/qr

# Login with phone number
POST /api/v1/auth/phone
{"phone": "+1234567890"}

# Logout
POST /api/v1/auth/logout
```

### Messaging

```bash
# Send text message
POST /api/v1/messages
{"phone": "+1234567890", "message": "Hello!"}

# Send file with caption
POST /api/v1/messages
{"phone": "+1234567890", "message": "Check this out", "file_path": "/path/to/image.jpg"}

# List chats
GET /api/v1/chats

# Get chat messages
GET /api/v1/chats/:chat_id

# Watch messages (SSE stream)
GET /api/v1/chats/events
```

### Health

```bash
GET /health          # Health check
GET /ready           # Readiness probe (K8s)
GET /live            # Liveness probe (K8s)
GET /metrics         # Service metrics
```

### Examples

```bash
# Send a message
curl -X POST http://localhost:3000/api/v1/messages \
  -H "Authorization: Bearer your-api-token" \
  -H "Content-Type: application/json" \
  -d '{"phone": "+1234567890", "message": "Hello from WAS!"}'

# Watch for new messages
curl -N http://localhost:3000/api/v1/chats/events \
  -H "Authorization: Bearer your-api-token"
```

**Swagger UI**: http://localhost:3000/swagger-ui/

## MCP (Model Context Protocol)

WAS implements MCP for AI agent integration. Works with Claude Desktop, Cursor, and other MCP clients.

### Available Tools

| Tool | Description |
|------|-------------|
| `whatsapp_get_auth_status` | Check WhatsApp connection status |
| `whatsapp_get_qr_code` | Get QR code for device linking |
| `whatsapp_login_with_phone` | Request phone pairing code |
| `whatsapp_logout` | Disconnect WhatsApp session |
| `whatsapp_send_message` | Send text or file message |
| `whatsapp_health_check` | Check service health |

### Claude Desktop Configuration

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "whatsapp": {
      "command": "npx",
      "args": ["-y", "mcp-remote", "http://localhost:3000/mcp"]
    }
  }
}
```

### MCP Endpoints

```bash
GET  /mcp              # SSE event stream
POST /mcp              # Send JSON-RPC messages
DELETE /mcp            # Terminate session
```

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
api_token = "your-secure-api-token"

[mcp]
enabled = true
endpoint = "/mcp"

[swagger]
enabled = true
path = "/swagger-ui"

[webhooks]
enabled = false

[[webhooks.endpoints]]
url = "https://your-server.com/webhook"
secret = "hmac-secret"
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `WHATSAPP_HOST` | Server host | `0.0.0.0` |
| `WHATSAPP_PORT` | Server port | `3000` |
| `WHATSAPP_API_TOKEN` | API bearer token | - |
| `RUST_LOG` | Log level | `info` |

## Webhooks

WAS pushes incoming messages to configured endpoints with HMAC-SHA256 signatures.

### Payload

```json
{
  "event": "message.received",
  "timestamp": "2026-02-19T10:30:00Z",
  "data": {
    "id": "msg-abc123",
    "sender": "+1234567890",
    "text": "Hello!",
    "is_group": false
  }
}
```

### Signature Verification

Each webhook request includes an `X-Webhook-Signature` header containing an HMAC-SHA256 signature. Verify it like this:

```python
import hmac
import hashlib

def verify_signature(payload: bytes, signature: str, secret: str) -> bool:
    expected = hmac.new(secret.encode(), payload, hashlib.sha256).hexdigest()
    return hmac.compare_digest(f"sha256={expected}", signature)
```

```javascript
const crypto = require('crypto');

function verifySignature(payload, signature, secret) {
  const expected = 'sha256=' + crypto.createHmac('sha256', secret).update(payload).digest('hex');
  return crypto.timingSafeEqual(Buffer.from(expected), Buffer.from(signature));
}
```

## Docker

```bash
# Development
docker-compose -f docker/docker-compose.yml up

# Production
docker-compose -f docker/docker-compose.production.yml up
```

## Project Structure

```
src/
├── lib.rs              # Library entry point
├── main.rs             # Alternative entry
├── error.rs            # Error types
├── session.rs          # Session management
├── bin/
│   └── whatsapp-server.rs  # Server binary
├── config/             # Configuration
├── handlers/           # HTTP handlers
│   ├── auth.rs         # Auth endpoints
│   ├── chat.rs         # Chat endpoints
│   ├── health.rs       # Health endpoint
│   └── mcp.rs          # MCP handlers
├── services/           # Business logic
│   ├── whatsapp.rs     # Main service
│   ├── auth_service.rs # Authentication
│   ├── chat_service.rs # Chat/messaging
│   ├── webhook.rs      # Webhook notifications
│   └── browser.rs      # Browser automation
├── models/             # Data models
└── utils/              # Utilities
```

## License

MIT License - see [LICENSE](LICENSE)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)

---

**Built with ❤️ by Devstroop Technologies**
