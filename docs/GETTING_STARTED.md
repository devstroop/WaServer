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

## Your First WhatsApp Integration

### Step 1: Create an Instance

```bash
curl -X POST http://localhost:3000/api/v1/instances \
  -H "Authorization: Bearer change-this-secret-key-in-production" \
  -H "Content-Type: application/json" \
  -d '{"name": "my-whatsapp"}'
```

Save the `instance_id` from the response.

### Step 2: Link Your WhatsApp

```bash
curl -X GET http://localhost:3000/api/v1/instances/{instance_id}/link/qr \
  -H "Authorization: Bearer change-this-secret-key-in-production" \
  --output qr.png
```

Scan the QR code with WhatsApp (Settings → Linked Devices → Link a Device).

### Step 3: Check Status

```bash
curl http://localhost:3000/api/v1/instances/{instance_id}/status \
  -H "Authorization: Bearer change-this-secret-key-in-production"
```

### Step 4: Send a Message

```bash
curl -X POST http://localhost:3000/api/v1/instances/{instance_id}/messages \
  -H "Authorization: Bearer change-this-secret-key-in-production" \
  -H "Content-Type: application/json" \
  -d '{"phone": "+1234567890", "message": "Hello from WAS!"}'
```

## Common Operations

### List All Instances
```bash
curl http://localhost:3000/api/v1/instances \
  -H "Authorization: Bearer change-this-secret-key-in-production"
```

### Send a File
```bash
curl -X POST http://localhost:3000/api/v1/instances/{instance_id}/messages \
  -H "Authorization: Bearer change-this-secret-key-in-production" \
  -F "phone=+1234567890" \
  -F "message=Check this out!" \
  -F "file=@/path/to/image.jpg"
```

### Delete an Instance
```bash
curl -X DELETE http://localhost:3000/api/v1/instances/{instance_id} \
  -H "Authorization: Bearer change-this-secret-key-in-production"
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
