# WAS - WhatsApp Server 🚀

A high-performance WhatsApp Web automation server built in Rust. Provides REST API and MCP (Model Context Protocol) interfaces for WhatsApp messaging.

## Features

- **Dual Authentication** - QR code and phone number authentication
- **REST API** - Full CRUD endpoints with OpenAPI/Swagger docs
- **MCP Server** - Model Context Protocol over Streamable HTTP (spec 2025-06-18)
- **Real-time Events** - SSE streams for message watching
- **Webhooks** - Push notifications for incoming messages
- **Modular Build** - Feature flags for lean deployments

## Quick Start

### Prerequisites

- Rust 1.70+ 
- Chrome/Chromium browser
- Windows, Linux, or macOS

### Install & Run

```bash
# Clone repository
git clone https://github.com/devstroop/was.git
cd was

# Copy config
cp config/app.example.toml config/app.toml

# Build (REST API included by default)
cargo build --release

# Build with MCP support
cargo build --release --features mcp

# Run
cargo run
```

Server starts at `http://localhost:3000`

## API Reference

### Authentication

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/auth/status` | Get authentication status |
| `GET` | `/api/v1/auth/qr` | Get QR code for authentication |
| `POST` | `/api/v1/auth/login` | Login with phone number |
| `POST` | `/api/v1/auth/logout` | Logout and clear session |

### Chats

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/chats` | List all chats |
| `GET` | `/api/v1/chats/:id` | Get messages for a chat |
| `GET` | `/api/v1/chats/events` | SSE stream for real-time messages |

### Messages

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/v1/messages` | Send a message |
| `GET` | `/api/v1/messages/:id` | Get message by ID |

### MCP (Model Context Protocol)

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/mcp` | SSE event stream |
| `POST` | `/mcp` | Send MCP messages |
| `DELETE` | `/mcp` | Terminate session |

### Health

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Health check |

## Usage Examples

### Send a Message

```bash
curl -X POST http://localhost:3000/api/v1/messages \
  -H "Authorization: Bearer your-api-token" \
  -H "Content-Type: application/json" \
  -d '{"phone": "+1234567890", "message": "Hello from WAS!"}'
```

### Check Auth Status

```bash
curl http://localhost:3000/api/v1/auth/status \
  -H "Authorization: Bearer your-api-token"
```

### Get QR Code

```bash
curl http://localhost:3000/api/v1/auth/qr \
  -H "Authorization: Bearer your-api-token"
```

### Login with Phone

```bash
curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Authorization: Bearer your-api-token" \
  -H "Content-Type: application/json" \
  -d '{"phone": "+1234567890"}'
```

### Watch Messages (SSE)

```bash
curl -N http://localhost:3000/api/v1/chats/events \
  -H "Authorization: Bearer your-api-token"
```

## Configuration

Edit `config/app.toml`:

```toml
[server]
host = "0.0.0.0"
port = 3000

[browser]
headless = true
timeout_ms = 30000

[auth]
api_token = "your-secure-api-token"

[logging]
level = "info"

[mcp]
enabled = true
endpoint = "/mcp"
sse_enabled = true
heartbeat_interval_secs = 30

[limits]
max_concurrent_requests = 50
request_timeout_ms = 30000
max_upload_size = 10485760
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `WHATSAPP_HOST` | Server host |
| `WHATSAPP_PORT` | Server port |
| `WHATSAPP_API_TOKEN` | API authentication token |

## Feature Flags

Build with optional features:

```bash
# Default build (REST API server)
cargo build --release

# With MCP server support
cargo build --release --features mcp
```

## Server Usage

```bash
# Run with defaults
was

# Run with environment variables
WHATSAPP_PORT=8080 WHATSAPP_API_TOKEN=secret was
```

## MCP Tools

When MCP is enabled, these tools are available:

| Tool | Description |
|------|-------------|
| `whatsapp_get_auth_status` | Check authentication status |
| `whatsapp_get_qr_code` | Get QR code for authentication |
| `whatsapp_login_with_phone` | Login with phone number |
| `whatsapp_logout` | Logout from WhatsApp |
| `whatsapp_send_message` | Send a message |
| `whatsapp_health_check` | Check service health |

### MCP Configuration (Claude Desktop)

Add to your Claude Desktop config:

```json
{
  "mcpServers": {
    "was": {
      "command": "npx",
      "args": ["-y", "mcp-remote", "http://localhost:3000/mcp"]
    }
  }
}
```

## API Documentation

Swagger UI available at: `http://localhost:3000/swagger-ui/`

OpenAPI spec at: `http://localhost:3000/api-docs/openapi.json`

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
