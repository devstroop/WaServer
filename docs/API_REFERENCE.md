# API Reference

Complete REST API documentation for WAS (WhatsApp Server).

**Base URL:** `http://localhost:3000`  
**Swagger UI:** `http://localhost:3000/api-docs/`

---

## Authentication

All protected endpoints require a Bearer token:

```http
Authorization: Bearer your-secret-key
```

Set the key in `config/app.toml`:
```toml
[api]
secret_key = "your-secret-key"
```

---

## Health Endpoints

Health endpoints do not require authentication.

### Health Check

```http
GET /api/health
```

**Response:**
```json
{
  "status": "healthy",
  "timestamp": 1709312400,
  "version": "0.3.0",
  "uptime_seconds": 3600,
  "instances_count": 2,
  "services": {
    "server": {
      "status": "healthy",
      "last_check": 1709312400,
      "response_time_ms": 0
    }
  }
}
```

### Readiness Probe

```http
GET /api/ready
```

**Response:**
```json
{"status": "ready"}
```

### Liveness Probe

```http
GET /api/live
```

**Response:**
```json
{"status": "live"}
```

### Metrics

```http
GET /api/metrics
```

**Response:**
```json
{
  "timestamp": 1709312400,
  "uptime_seconds": 3600,
  "memory_usage_bytes": 52428800,
  "instances_count": 2,
  "instances": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "status": "active",
      "authorized": true,
      "total_messages_sent": 150,
      "error_count": 2
    }
  ]
}
```

---

## Instance Management

### List Instances

```http
GET /api/v1/instances
```

**Response:**
```json
{
  "instances": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "sales-bot",
      "phone_number": "+1234567890",
      "status": "active",
      "authorized": true,
      "created_at": "2026-03-01T10:00:00Z",
      "updated_at": "2026-03-01T12:00:00Z"
    }
  ],
  "total": 1
}
```

### Create Instance

```http
POST /api/v1/instances
Content-Type: application/json

{
  "name": "my-whatsapp",
  "phone_number": "+1234567890"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | string | Yes | Friendly name for the instance |
| phone_number | string | No | Phone number in E.164 format (for phone linking) |

**Response (201 Created):**
```json
{
  "instance_id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "my-whatsapp",
  "message": "Instance created successfully"
}
```

**Errors:**
- `400` - Invalid request (bad phone format)
- `409` - Instance already exists

### Get Instance

```http
GET /api/v1/instances/{instance_id}
```

| Parameter | Type | Description |
|-----------|------|-------------|
| instance_id | string | UUID or phone number |

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "my-whatsapp",
  "phone_number": "+1234567890",
  "status": "active",
  "authorized": true,
  "created_at": "2026-03-01T10:00:00Z",
  "updated_at": "2026-03-01T12:00:00Z"
}
```

### Delete Instance

```http
DELETE /api/v1/instances/{instance_id}?delete_data=true
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| instance_id | path | - | UUID or phone number |
| delete_data | query | false | Delete all instance data (sessions, chrome profile) |

**Response:**
```json
{
  "message": "Instance and all data deleted",
  "instance_id": "550e8400-e29b-41d4-a716-446655440000",
  "data_deleted": true
}
```

### Warmup Instance

Pre-warm an instance's browser for faster subsequent requests.

```http
POST /api/v1/instances/{instance_id}/warmup
```

**Response:**
```json
{
  "message": "Instance warmed up successfully",
  "instance_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Errors:**
- `409` - Instance already warming up

### Get Screenshot

Get a live screenshot of the instance's browser.

```http
GET /api/v1/instances/{instance_id}/screenshot
```

**Response:** `image/png` binary data

### Get Instance Config

```http
GET /api/v1/instances/{instance_id}/config
```

**Response:**
```json
{
  "idle_timeout_seconds": 300,
  "warmup_timeout_seconds": 60,
  "webhook_url": "https://example.com/webhook",
  "headless": true
}
```

### Update Instance Config

```http
PUT /api/v1/instances/{instance_id}/config
Content-Type: application/json

{
  "idle_timeout_seconds": 600,
  "webhook_url": "https://example.com/new-webhook"
}
```

**Response:**
```json
{
  "message": "Configuration updated successfully",
  "config": {...},
  "restart_required": true
}
```

### Reset Instance

Wipe all session data without deleting the instance.

```http
DELETE /api/v1/instances/{instance_id}/reset
```

**Response:**
```json
{
  "message": "Instance reset — all session data cleared",
  "instance_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

---

## WhatsApp Operations

### Get Status

```http
GET /api/v1/instances/{instance_id}/status
```

**Response:**
```json
{
  "instance_id": "550e8400-e29b-41d4-a716-446655440000",
  "phone_number": "+1234567890",
  "status": "active",
  "authorized": true
}
```

**Status values:**
- `sleeping` - Browser not running
- `warming_up` - Browser starting
- `active` - Ready for operations
- `error` - Instance has an error

### Get QR Code

Get QR code image for WhatsApp Web linking.

```http
GET /api/v1/instances/{instance_id}/link/qr
```

**Response:** `image/png` QR code image

**Errors:**
- `409` - Already authorized (call `/unlink` first)
- `503` - Browser failed to start

### Link with Phone Number

Initiate phone number linking (uses phone set during instance creation).

```http
POST /api/v1/instances/{instance_id}/link/phone
```

**Response:**
```json
{
  "success": true,
  "phone_number": "+1234567890",
  "linking_code": "ABC-DEFGH"
}
```

Enter the `linking_code` on your phone to complete linking.

**Errors:**
- `400` - No phone number configured
- `409` - Already authorized

### Unlink (Logout)

Disconnect WhatsApp Web session.

```http
DELETE /api/v1/instances/{instance_id}/unlink
```

**Response:**
```json
{
  "success": true,
  "message": "WhatsApp Web session unlinked"
}
```

---

## Messaging

### Send Message

Send text message and/or file attachment.

```http
POST /api/v1/instances/{instance_id}/send?phone=+1234567890&text=Hello!
```

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| phone | string | Yes | Recipient phone in E.164 format |
| text | string | No* | Message text or caption |

*At least one of `text` or file attachment required.

**Send with file attachment (multipart):**

```http
POST /api/v1/instances/{instance_id}/send?phone=+1234567890&text=Check this out
Content-Type: multipart/form-data

--boundary
Content-Disposition: form-data; name="file"; filename="image.jpg"
Content-Type: image/jpeg

<binary data>
--boundary--
```

**Response:**
```json
{
  "success": true,
  "message_id": "3EB0ABC123",
  "phone": "+1234567890",
  "timestamp": "2026-03-01T12:30:00Z"
}
```

**Errors:**
- `400` - Missing phone or no text/file provided
- `401` - Instance not authorized (scan QR first)
- `503` - Instance busy or browser unavailable

### List Chats

```http
GET /api/v1/instances/{instance_id}/chats
```

**Response:**
```json
{
  "chats": [
    {
      "name": "John Doe",
      "phone": "+1234567890",
      "last_message": "Thanks!",
      "timestamp": "2026-03-01T12:00:00Z",
      "unread_count": 2
    }
  ],
  "total": 1
}
```

### Get Messages

```http
GET /api/v1/instances/{instance_id}/messages/{phone}?limit=50&load_more=false
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| limit | integer | 50 | Max messages to retrieve |
| load_more | boolean | false | Load older messages by scrolling |

**Response:**
```json
{
  "messages": [
    {
      "id": "3EB0ABC123",
      "from": "+1234567890",
      "to": "me",
      "text": "Hello there!",
      "timestamp": "2026-03-01T12:00:00Z",
      "is_outgoing": false,
      "status": "read"
    }
  ],
  "total": 1,
  "phone": "+1234567890"
}
```

---

## Server-Sent Events (SSE)

Subscribe to real-time events from an instance.

```http
GET /api/v1/instances/{instance_id}/events
Accept: text/event-stream
```

**Event types:**
- `message` - New incoming message
- `status` - Instance status change
- `qr_updated` - New QR code available
- `authorized` - Successfully linked

**Example event:**
```
event: message
data: {"from": "+1234567890", "text": "Hello!", "timestamp": "2026-03-01T12:00:00Z"}

```

---

## Error Responses

All error responses follow this format:

```json
{
  "error": "error_code",
  "message": "Human readable description"
}
```

### Error Codes

| Code | HTTP | Description |
|------|------|-------------|
| instance_not_found | 404 | Instance doesn't exist |
| already_authorized | 409 | Already linked to WhatsApp |
| not_authorized | 401 | Not linked - scan QR first |
| warmup_failed | 503 | Browser failed to start |
| browser_timeout | 503 | Operation timed out |
| instance_busy | 503 | Another operation in progress |
| auth_check_failed | 500 | Failed to check auth status |
| qr_failed | 500 | Failed to generate QR code |

---

## Rate Limits

Default limits (configurable):
- 100 requests/minute per instance
- 10 concurrent requests per instance

---

## Examples

### cURL

```bash
# Create instance
curl -X POST http://localhost:3000/api/v1/instances \
  -H "Authorization: Bearer secret" \
  -H "Content-Type: application/json" \
  -d '{"name": "bot", "phone_number": "+1234567890"}'

# Get QR code
curl http://localhost:3000/api/v1/instances/{id}/link/qr \
  -H "Authorization: Bearer secret" \
  -o qr.png

# Send message
curl -X POST "http://localhost:3000/api/v1/instances/{id}/send?phone=+1234567890&text=Hello" \
  -H "Authorization: Bearer secret"

# Send file
curl -X POST "http://localhost:3000/api/v1/instances/{id}/send?phone=+1234567890&text=Caption" \
  -H "Authorization: Bearer secret" \
  -F "file=@image.jpg"
```

### Python

```python
import requests

BASE = "http://localhost:3000"
HEADERS = {"Authorization": "Bearer secret"}

# Create instance
r = requests.post(f"{BASE}/api/v1/instances", 
    headers=HEADERS, 
    json={"name": "bot", "phone_number": "+1234567890"})
instance_id = r.json()["instance_id"]

# Check status
status = requests.get(f"{BASE}/api/v1/instances/{instance_id}/status", 
    headers=HEADERS).json()
print(f"Status: {status['status']}, Authorized: {status['authorized']}")

# Get QR code
r = requests.get(f"{BASE}/api/v1/instances/{instance_id}/link/qr", 
    headers=HEADERS)
with open("qr.png", "wb") as f:
    f.write(r.content)

# Send message
requests.post(f"{BASE}/api/v1/instances/{instance_id}/send",
    headers=HEADERS,
    params={"phone": "+1234567890", "text": "Hello from Python!"})

# Send file
with open("image.jpg", "rb") as f:
    requests.post(f"{BASE}/api/v1/instances/{instance_id}/send",
        headers=HEADERS,
        params={"phone": "+1234567890", "text": "Check this out"},
        files={"file": f})
```

### JavaScript

```javascript
const BASE = "http://localhost:3000";
const headers = { 
  "Authorization": "Bearer secret", 
  "Content-Type": "application/json" 
};

// Create instance
const { instance_id } = await fetch(`${BASE}/api/v1/instances`, {
  method: "POST",
  headers,
  body: JSON.stringify({ name: "bot", phone_number: "+1234567890" })
}).then(r => r.json());

// Send message
await fetch(`${BASE}/api/v1/instances/${instance_id}/send?phone=+1234567890&text=Hello`, {
  method: "POST",
  headers: { "Authorization": "Bearer secret" }
});

// Subscribe to events
const events = new EventSource(
  `${BASE}/api/v1/instances/${instance_id}/events`,
  { headers: { "Authorization": "Bearer secret" } }
);

events.onmessage = (e) => {
  console.log("Event:", JSON.parse(e.data));
};
```
