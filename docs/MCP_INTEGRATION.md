# MCP Integration Guide

WAS implements the Model Context Protocol (MCP) for AI agent integration with Claude, Cursor, and other MCP clients.

## Prerequisites

1. Build with MCP support:
   ```bash
   cargo build --release --features mcp
   ```

2. Enable in config:
   ```toml
   [mcp]
   enabled = true
   endpoint = "/mcp"
   ```

## Available Tools

| Tool | Description |
|------|-------------|
| `whatsapp_get_auth_status` | Check connection status |
| `whatsapp_get_qr_code` | Get QR code for linking |
| `whatsapp_login_with_phone` | Request pairing code |
| `whatsapp_logout` | Disconnect session |
| `whatsapp_send_message` | Send text message |
| `whatsapp_health_check` | Check service health |

## Claude Desktop Setup

Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS):

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

Windows: `%APPDATA%\Claude\claude_desktop_config.json`

Restart Claude Desktop after making changes.

## Cursor IDE

Add to Cursor settings:
```json
{
  "mcp.servers": {
    "whatsapp": {
      "command": "npx",
      "args": ["-y", "mcp-remote", "http://localhost:3000/mcp"]
    }
  }
}
```

## MCP Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/mcp` | GET | SSE event stream |
| `/mcp` | POST | JSON-RPC messages |
| `/mcp` | DELETE | Terminate session |

## Usage Examples

### Send Message via Claude

> "Send a WhatsApp message to +1234567890 saying 'Hello!'"

### Check Status

> "Is my WhatsApp connected?"

### Link New Device

> "I need to connect a new WhatsApp account"

## Tool Parameters

### whatsapp_send_message
```json
{
  "instance_id": "uuid",
  "phone": "+1234567890",
  "message": "Hello!"
}
```

### whatsapp_get_auth_status
```json
{
  "instance_id": "uuid"
}
```

## Error Codes

| Code | Description |
|------|-------------|
| -32000 | Instance not found |
| -32001 | Not authorized |
| -32002 | Service unavailable |
| -32003 | Send failed |

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Cannot connect | Verify `--features mcp` and config |
| Tool not found | Restart Claude/Cursor |
| Instance not found | Create instance via REST API first |
