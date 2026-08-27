# Getting Started with WAS

This guide walks you through installing, running, and using WAS for the first time.

## Prerequisites

### Required Software

| Software | Version | Purpose |
|----------|---------|---------|
| **Rust** | 1.70+ | Compile and run WAS |
| **Chrome/Chromium** | Latest | Browser automation |
| **Git** | Any | Clone repository |

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version
```

### Install Chrome/Chromium

**Ubuntu/Debian:**
```bash
sudo apt update && sudo apt install chromium-browser
```

**macOS:**
```bash
brew install --cask chromium
```

## Installation

### Option 1: Build from Source

```bash
git clone https://github.com/devstroop/was.git
cd was
cp config/app.example.toml config/app.toml
cargo build --release
cargo run --release
```

### Option 2: Docker

```bash
git clone https://github.com/devstroop/was.git
cd was
docker-compose up -d
```

## First Run

Server starts at **http://localhost:3000**

- **API**: http://localhost:3000
- **Swagger UI**: http://localhost:3000/api-docs/
- **Health**: http://localhost:3000/api/health

### Browser requirement

Sending, linking (QR) and screenshots require **Chrome or Chromium** on the server host.
The server detects it at startup and logs a warning when missing; `GET /api/health`
reports `browser_available`.

```bash
# macOS
brew install --cask chromium
# Debian/Ubuntu
sudo apt install chromium-browser
# Or point at any Chrome-compatible binary:
export CHROME=/path/to/chrome
```

### Authentication (opt-in)

There is **no default API key**. Two ways to authenticate:

1. **Static superadmin key** — set it in `config/app.toml` (`[auth] secret_key`, ≥16 chars),
   then use `Authorization: Bearer <key>`:
   ```bash
   export WAS_API_KEY="set-a-long-random-secret-here"
   ```
2. **User access tokens** — register a user, log in (`POST /api/v1/auth/login`),
   or mint tokens via the API (`POST /api/v1/users/:id/tokens`).

If no `secret_key` is configured, the static-key path is disabled entirely and
user tokens are the only way in.

## Your First WhatsApp Integration

### Step 1: Create an Instance

```bash
curl -X POST http://localhost:3000/api/v1/instances \
  -H "Authorization: Bearer $WAS_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"name": "my-whatsapp"}'
```

Save the `instance_id` from the response.

### Step 2: Link Your WhatsApp

```bash
curl -X GET http://localhost:3000/api/v1/instances/{instance_id}/link/qr \
  -H "Authorization: Bearer $WAS_API_KEY" \
  --output qr.png
```

Scan the QR code with WhatsApp (Settings → Linked Devices → Link a Device).

### Step 3: Check Status

```bash
curl http://localhost:3000/api/v1/instances/{instance_id}/status \
  -H "Authorization: Bearer $WAS_API_KEY"
```

### Step 4: Send a Message

```bash
curl -X POST http://localhost:3000/api/v1/instances/{instance_id}/messages \
  -H "Authorization: Bearer $WAS_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"phone": "+1234567890", "message": "Hello from WAS!"}'
```

## Common Operations

### List All Instances
```bash
curl http://localhost:3000/api/v1/instances \
  -H "Authorization: Bearer $WAS_API_KEY"
```

### Send a File
```bash
curl -X POST http://localhost:3000/api/v1/instances/{instance_id}/messages \
  -H "Authorization: Bearer $WAS_API_KEY" \
  -F "phone=+1234567890" \
  -F "message=Check this out!" \
  -F "file=@/path/to/image.jpg"
```

### Delete an Instance
```bash
curl -X DELETE http://localhost:3000/api/v1/instances/{instance_id} \
  -H "Authorization: Bearer $WAS_API_KEY"
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Browser not found | Install Chrome/Chromium |
| Connection refused | Check if server is running |
| QR code not loading | Run with `RUST_LOG=debug` |
| Auth timeout | QR codes expire in ~60 seconds |

## Next Steps

- [API Reference](API_REFERENCE.md) - Full endpoint documentation
- [Configuration](CONFIGURATION.md) - All config options
