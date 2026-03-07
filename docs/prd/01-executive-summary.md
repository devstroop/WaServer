# 1. Executive Summary

## Product Overview

**WAS (WhatsApp Server)** is a high-performance, multi-instance WhatsApp Web automation platform built in Rust. It provides a robust REST API and MCP (Model Context Protocol) interface for programmatic WhatsApp messaging, enabling businesses and developers to integrate WhatsApp communication into their applications.

## Value Proposition

| Stakeholder | Value |
|-------------|-------|
| **Businesses** | Automate customer communication at scale |
| **Developers** | Simple REST API for WhatsApp integration |
| **AI Platforms** | MCP support for AI agent workflows |
| **Operations** | Multi-instance management from single server |

## Key Differentiators

1. **Multi-Instance Architecture** - Run hundreds of WhatsApp accounts from one server
2. **No Official API Required** - Uses WhatsApp Web automation (no Meta Business API needed)
3. **MCP Protocol Support** - First-class integration with AI assistants like Claude
4. **High Performance** - Built in Rust with async runtime for maximum efficiency
5. **Self-Hosted** - Full data control, no third-party dependencies

## Target Market

- Small to medium businesses needing WhatsApp automation
- SaaS platforms requiring messaging capabilities
- AI/chatbot developers building conversational interfaces
- Marketing automation platforms
- Customer support platforms

## Technology Stack

| Component | Technology |
|-----------|------------|
| Language | Rust 1.70+ |
| Runtime | Tokio (async) |
| Web Framework | Axum 0.7 |
| Browser Automation | chromiumoxide (CDP) |
| Database | SQLite (rusqlite) |
| Documentation | utoipa + Swagger UI |

## Quick Start

```bash
# Run with Docker
docker run -d -p 3000:3000 ghcr.io/devstroop/was:latest

# Create instance
curl -X POST http://localhost:3000/api/v1/instances \
  -H "Content-Type: application/json" \
  -d '{"instance_id": "my-instance"}'

# Get QR code and scan with WhatsApp
curl http://localhost:3000/api/v1/whatsapp/my-instance/qrcode
```

---

[Next: Problem Statement →](02-problem-statement.md)
