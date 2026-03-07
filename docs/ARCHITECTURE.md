# Architecture Overview

## System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      WAS Server                          │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐           │
│  │  REST API │  │MCP Server │  │ Webhooks  │           │
│  │  (Axum)   │  │  (SSE)    │  │  (HTTP)   │           │
│  └─────┬─────┘  └─────┬─────┘  └─────┬─────┘           │
│        └──────────────┼──────────────┘                  │
│                       ▼                                  │
│  ┌───────────────────────────────────────────────────┐  │
│  │              Instance Manager                      │  │
│  │  ┌────────┐  ┌────────┐  ┌────────┐               │  │
│  │  │Instance│  │Instance│  │Instance│               │  │
│  │  │   #1   │  │   #2   │  │   #3   │               │  │
│  │  └───┬────┘  └───┬────┘  └───┬────┘               │  │
│  └──────┼───────────┼───────────┼────────────────────┘  │
│         └───────────┴───────────┘                       │
│                     ▼                                    │
│  ┌───────────────────────────────────────────────────┐  │
│  │            Browser Pool (Chromium)                 │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐         │  │
│  │  │ Browser  │  │ Browser  │  │ Browser  │         │  │
│  │  │ (WA Web) │  │ (WA Web) │  │ (WA Web) │         │  │
│  │  └──────────┘  └──────────┘  └──────────┘         │  │
│  └───────────────────────────────────────────────────┘  │
│  ┌───────────┐  ┌───────────┐                          │
│  │ Database  │  │  Config   │                          │
│  │ (SQLite)  │  │  (TOML)   │                          │
│  └───────────┘  └───────────┘                          │
└─────────────────────────────────────────────────────────┘
```

## Core Components

### HTTP Server (Axum)
- Async request handling with Tokio
- OpenAPI/Swagger documentation
- CORS and middleware support

### Instance Manager
- Multi-account support
- Instance lifecycle (sleep/wake)
- Request routing

### Browser Service
- Chromium automation via chromiumoxide
- Session persistence
- Dynamic element locators

## Project Structure

```
src/
├── bin/was.rs           # Entry point
├── handlers/api/        # REST API handlers
│   ├── instances.rs     # Instance management
│   ├── whatsapp.rs      # WhatsApp operations
│   ├── chat.rs          # Messaging
│   ├── health.rs        # Health checks
│   └── mcp.rs           # MCP protocol
├── services/            # Business logic
│   ├── whatsapp/        # WhatsApp service
│   ├── auth/            # Authentication
│   ├── database/        # SQLite operations
│   └── webhook.rs       # Webhook delivery
├── browser/             # Browser automation
│   ├── core.rs          # Browser lifecycle
│   ├── driver.rs        # Chromium driver
│   ├── session.rs       # Session management
│   └── locators.rs      # Element selectors
├── models/              # Data types
├── middleware/          # HTTP middleware
├── config.rs            # Configuration
└── error.rs             # Error types
```

## Instance Lifecycle

```
SLEEPING ──request──► WARMING UP ──ready──► ACTIVE
    ▲                                          │
    └────────────idle timeout──────────────────┘
                                               │
                                        error  ▼
                                            ERROR
```

## Request Flow

1. Request arrives at HTTP server
2. Auth middleware validates token
3. Router dispatches to handler
4. Handler gets instance from manager
5. Instance warms up if sleeping
6. Browser executes operation
7. Response returned to client

## Database Schema

```sql
CREATE TABLE instances (
    id TEXT PRIMARY KEY,
    name TEXT,
    phone_number TEXT,
    created_at TEXT,
    updated_at TEXT
);

CREATE TABLE sessions (
    instance_id TEXT PRIMARY KEY,
    data BLOB,
    updated_at TEXT
);
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| default | REST API only |
| mcp | MCP server support |
