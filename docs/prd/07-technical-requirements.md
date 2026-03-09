# 7. Technical Requirements

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Client Applications                       │
└─────────────────────────────────────────────────────────────────┘
                              │
                    HTTPS / REST API / MCP
                              │
┌─────────────────────────────────────────────────────────────────┐
│                         WAS Server                               │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐        │
│  │  HTTP Server  │  │  MCP Server   │  │   Webhooks    │        │
│  │    (Axum)     │  │  (SSE/JSON)   │  │   Manager     │        │
│  └───────────────┘  └───────────────┘  └───────────────┘        │
│                             │                                    │
│  ┌──────────────────────────▼───────────────────────────┐       │
│  │                  Service Layer                        │       │
│  │  Instance Manager │ Chat Service │ Auth Service       │       │
│  └──────────────────────────────────────────────────────┘       │
│                             │                                    │
│  ┌──────────────────────────▼───────────────────────────┐       │
│  │                  Browser Layer                        │       │
│  │  Browser Service │ WhatsApp Engine │ Session Manager  │       │
│  └──────────────────────────────────────────────────────┘       │
│                             │                                    │
│  ┌──────────────────────────▼───────────────────────────┐       │
│  │                  Data Layer                           │       │
│  │  SQLite Database │ Chrome Profiles │ Config Files     │       │
│  └──────────────────────────────────────────────────────┘       │
└─────────────────────────────────────────────────────────────────┘
```

## Technology Stack

| Component | Technology | Version |
|-----------|------------|---------|
| Language | Rust | 1.70+ |
| Runtime | Tokio | 1.0+ |
| Web Framework | Axum | 0.7 |
| Browser Automation | chromiumoxide | 0.5 |
| Database | SQLite (rusqlite) | 0.31 |
| Serialization | Serde | 1.0 |
| Logging | Tracing | 0.1 |
| Documentation | utoipa | 4.0 |

## Environment Requirements

### Production

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| OS | Ubuntu 20.04+ | Ubuntu 22.04 |
| CPU | 2 cores | 4+ cores |
| Memory | 2 GB | 4+ GB |
| Disk | 10 GB SSD | 50+ GB SSD |

## Configuration

```toml
[server]
host = "0.0.0.0"
port = 3000

[auth]
enabled = true
secret_key = "env:WAS__AUTH__SECRET_KEY"

[browser]
executable_path = "/usr/bin/chromium"
headless = true

[mcp]
enabled = true
endpoint = "/mcp"

[webhook]
enabled = false
url = ""
```

---

[← Previous: Use Cases](06-use-cases.md) | [Next: Non-Functional Requirements →](08-non-functional-requirements.md)
