# WAS Documentation

Welcome to the WAS (WhatsApp Server) documentation. This guide covers everything you need to know to install, configure, and use WAS.

## What is WAS?

WAS is a high-performance WhatsApp Web automation server built in Rust. It provides:

- **REST API** for sending and receiving WhatsApp messages
- **MCP Server** for AI agent integration (Claude, Cursor, etc.)
- **Multi-instance support** for managing multiple WhatsApp accounts
- **Webhook support** for real-time message notifications

## Documentation Index

| Document | Description |
|----------|-------------|
| [Getting Started](GETTING_STARTED.md) | Installation and first steps |
| [API Reference](API_REFERENCE.md) | Complete REST API documentation |
| [Configuration](CONFIGURATION.md) | All configuration options |
| [Deployment](DEPLOYMENT.md) | Production deployment guide |
| [MCP Integration](MCP_INTEGRATION.md) | AI agent integration guide |
| [Architecture](ARCHITECTURE.md) | System architecture overview |

## Quick Links

- **Swagger UI**: `http://localhost:3000/api-docs/` (when running)
- **Health Check**: `GET /api/health`
- **GitHub**: [devstroop/was](https://github.com/devstroop/was)

## Requirements

| Requirement | Version |
|-------------|---------|
| Rust | 1.70+ |
| Chrome/Chromium | Latest stable |
| OS | Linux, macOS, Windows |

## 5-Minute Quickstart

```bash
# Clone and configure
git clone https://github.com/devstroop/was.git
cd was
cp config/app.example.toml config/app.toml

# Build and run
cargo run --release

# Server starts at http://localhost:3000
```

Then:
1. Create an instance: `POST /api/v1/instances`
2. Get QR code: `GET /api/v1/instances/{id}/link/qr`
3. Scan with your phone
4. Send messages: `POST /api/v1/instances/{id}/messages`

## Need Help?

- Check the [Configuration](CONFIGURATION.md) guide for environment setup
- See [Deployment](DEPLOYMENT.md) for production best practices
- Review [API Reference](API_REFERENCE.md) for endpoint details
