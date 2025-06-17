# WhatsApp Engine - API Reference 📚

Complete API documentation for WhatsApp Engine REST API server.

## 🌐 Base Information

- **Base URL**: `http://localhost:3000` (development) / `https://your-domain.com` (production)
- **API Version**: `v1`
- **Content-Type**: `application/json`
- **Authentication**: Bearer token in `Authorization` header

## 🔐 Authentication

All API endpoints require authentication using Bearer token:

```http
Authorization: Bearer your-api-token-here
```

Configure your API token in:
- Environment variable: `AUTH_API_TOKEN`
- Configuration file: `config/app.toml` under `[auth]` section

## 📋 API Endpoints

### 🔐 Authentication Endpoints

#### Get Authentication Status
```http
GET /api/auth/status
```

**Description**: Check current WhatsApp authentication status

**Response**:
```json
{
  "authorized": true,
  "sender_id": "+1234567890",
  "connection_state": "CONNECTED",
  "session_active": true,
  "last_activity": "2024-01-15T10:30:00Z"
}
```

**Status Codes**:
- `200 OK`: Status retrieved successfully
- `401 Unauthorized`: Invalid API token
- `500 Internal Server Error`: Service error

---

#### Generate QR Code
```http
GET /api/auth/qrcode
```

**Description**: Generate QR code for WhatsApp Web authentication

**Response**:
```json
{
  "qrcode": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA...",
  "expires_at": "2024-01-15T10:35:00Z",
  "refresh_interval": 30
}
```

**Status Codes**:
- `200 OK`: QR code generated successfully
- `400 Bad Request`: Already authenticated
- `401 Unauthorized`: Invalid API token
- `500 Internal Server Error`: QR generation failed

---

#### Phone Authentication
```http
POST /api/auth/phone/{phone_number}
```

**Description**: Authenticate using phone number

**Parameters**:
- `phone_number` (path): Phone number in international format (+1234567890)

**Response**:
```json
{
  "success": true,
  "code": "1CBZ-9GEJ",
  "message": "Verification code sent to WhatsApp",
  "expires_in": 300
}
```

**Status Codes**:
- `200 OK`: Authentication initiated successfully
- `400 Bad Request`: Invalid phone number format
- `401 Unauthorized`: Invalid API token
- `409 Conflict`: Already authenticated
- `429 Too Many Requests`: Rate limited
- `500 Internal Server Error`: Authentication failed

---

#### Logout
```http
POST /api/auth/logout
```

**Description**: Logout from WhatsApp Web session

**Response**:
```json
{
  "success": true,
  "message": "Logged out successfully"
}
```

**Status Codes**:
- `200 OK`: Logged out successfully
- `401 Unauthorized`: Invalid API token
- `500 Internal Server Error`: Logout failed

---

### 💬 Chat Endpoints

#### Send Message
```http
POST /api/chat/send
```

**Description**: Send text message to WhatsApp contact

**Request Body**:
```json
{
  "to": "1234567890",
  "message": "Hello from WhatsApp Engine!"
}
```

**Response**:
```json
{
  "success": true,
  "message_id": "msg_123456789",
  "status": "sent",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

**Status Codes**:
- `200 OK`: Message sent successfully
- `400 Bad Request`: Invalid request format
- `401 Unauthorized`: Invalid API token or not authenticated
- `422 Unprocessable Entity`: Validation error
- `429 Too Many Requests`: Rate limited
- `500 Internal Server Error`: Send failed

---

#### Send File
```http
POST /api/chat/send-file
```

**Description**: Send file attachment to WhatsApp contact

**Request**: Multipart form data
- `to` (string): Recipient phone number
- `file` (file): File to send (max 16MB)
- `caption` (string, optional): File caption

**Response**:
```json
{
  "success": true,
  "message_id": "msg_123456789",
  "file_name": "document.pdf",
  "file_size": 1024000,
  "status": "sent",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

**Status Codes**:
- `200 OK`: File sent successfully
- `400 Bad Request`: Invalid request or file too large
- `401 Unauthorized`: Invalid API token or not authenticated
- `415 Unsupported Media Type`: File type not supported
- `422 Unprocessable Entity`: Validation error
- `500 Internal Server Error`: Send failed

---

### 👥 Contact Endpoints

#### Get Contacts
```http
GET /api/contacts
```

**Description**: Retrieve WhatsApp contacts list

**Query Parameters**:
- `limit` (optional): Number of contacts to return (default: 100, max: 1000)
- `offset` (optional): Pagination offset (default: 0)
- `search` (optional): Search query for contact names

**Response**:
```json
{
  "contacts": [
    {
      "id": "contact_123",
      "name": "John Doe",
      "phone": "+1234567890",
      "is_business": false,
      "profile_picture": "https://...",
      "last_seen": "2024-01-15T09:30:00Z"
    }
  ],
  "total": 150,
  "limit": 100,
  "offset": 0
}
```

**Status Codes**:
- `200 OK`: Contacts retrieved successfully
- `401 Unauthorized`: Invalid API token or not authenticated
- `500 Internal Server Error`: Retrieval failed

---

#### Get Chats
```http
GET /api/chats
```

**Description**: Retrieve WhatsApp chats list

**Query Parameters**:
- `limit` (optional): Number of chats to return (default: 50, max: 500)
- `offset` (optional): Pagination offset (default: 0)
- `unread_only` (optional): Show only unread chats (default: false)

**Response**:
```json
{
  "chats": [
    {
      "id": "chat_123",
      "name": "John Doe",
      "is_group": false,
      "last_message": "Hello there!",
      "last_message_time": "2024-01-15T10:25:00Z",
      "unread_count": 3,
      "participants": 2
    }
  ],
  "total": 75,
  "limit": 50,
  "offset": 0
}
```

**Status Codes**:
- `200 OK`: Chats retrieved successfully
- `401 Unauthorized`: Invalid API token or not authenticated
- `500 Internal Server Error`: Retrieval failed

---

### 🔍 Health & Monitoring Endpoints

#### Health Check
```http
GET /health
```

**Description**: Basic health check (no authentication required)

**Response**:
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "0.2.0"
}
```

---

#### Readiness Check
```http
GET /ready
```

**Description**: Readiness probe for orchestration (no authentication required)

**Response**:
```json
{
  "ready": true,
  "services": {
    "browser": "connected",
    "whatsapp": "loaded",
    "database": "connected"
  },
  "timestamp": "2024-01-15T10:30:00Z"
}
```

---

#### Liveness Check
```http
GET /live
```

**Description**: Liveness probe for orchestration (no authentication required)

**Response**:
```json
{
  "alive": true,
  "uptime_seconds": 3600,
  "timestamp": "2024-01-15T10:30:00Z"
}
```

---

#### Metrics
```http
GET /metrics
```

**Description**: Prometheus-style metrics (no authentication required)

**Response**: Plain text metrics format
```
# HELP whatsapp_engine_requests_total Total HTTP requests
# TYPE whatsapp_engine_requests_total counter
whatsapp_engine_requests_total{method="POST",endpoint="/api/chat/send"} 150

# HELP whatsapp_engine_messages_sent_total Total messages sent
# TYPE whatsapp_engine_messages_sent_total counter
whatsapp_engine_messages_sent_total 145

# HELP whatsapp_engine_uptime_seconds Service uptime in seconds
# TYPE whatsapp_engine_uptime_seconds gauge
whatsapp_engine_uptime_seconds 3600
```

---

## 🚨 Error Response Format

All errors follow a consistent format:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Phone number must be in international format",
    "details": {
      "field": "phone_number",
      "provided": "123456789",
      "expected": "+1234567890"
    }
  },
  "timestamp": "2024-01-15T10:30:00Z",
  "path": "/api/chat/send"
}
```

### Error Codes

| Code | Description |
|------|-------------|
| `AUTHENTICATION_REQUIRED` | Valid API token required |
| `NOT_AUTHENTICATED` | WhatsApp authentication required |
| `VALIDATION_ERROR` | Request validation failed |
| `RATE_LIMITED` | Too many requests |
| `SERVICE_UNAVAILABLE` | Service temporarily unavailable |
| `INTERNAL_ERROR` | Internal server error |

---

## 📊 Rate Limiting

API endpoints are rate limited to prevent abuse:

| Endpoint Pattern | Limit | Window |
|------------------|-------|--------|
| `/api/auth/*` | 10 requests | 1 minute |
| `/api/chat/send` | 30 requests | 1 minute |
| `/api/chat/send-file` | 10 requests | 1 minute |
| `/api/contacts` | 60 requests | 1 hour |
| `/api/chats` | 60 requests | 1 hour |

Rate limit headers are included in responses:
```http
X-RateLimit-Limit: 30
X-RateLimit-Remaining: 25
X-RateLimit-Reset: 1642248600
```

---

## 🔧 Configuration

### Environment Variables

```bash
# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Authentication
AUTH_API_TOKEN=your-secure-api-token

# Rate Limiting
RATE_LIMIT_ENABLED=true
RATE_LIMIT_REQUESTS_PER_MINUTE=30

# File Upload
MAX_FILE_SIZE_MB=16
ALLOWED_FILE_TYPES=pdf,jpg,jpeg,png,gif,doc,docx

# Browser Configuration
BROWSER_HEADLESS=true
BROWSER_TIMEOUT_MS=30000
```

### Configuration File

```toml
# config/app.toml
[server]
host = "0.0.0.0"
port = 3000

[auth]
api_token = "your-secure-api-token"

[limits]
max_file_size_bytes = 16777216  # 16MB
request_timeout_ms = 30000
rate_limit_requests_per_minute = 30

[browser]
headless = true
timeout_ms = 30000
```

---

## 📝 Usage Examples

### cURL Examples

#### Authentication Status
```bash
curl -H "Authorization: Bearer your-token" \
     http://localhost:3000/api/auth/status
```

#### Send Message
```bash
curl -X POST \
     -H "Authorization: Bearer your-token" \
     -H "Content-Type: application/json" \
     -d '{"to": "1234567890", "message": "Hello!"}' \
     http://localhost:3000/api/chat/send
```

#### Send File
```bash
curl -X POST \
     -H "Authorization: Bearer your-token" \
     -F "to=1234567890" \
     -F "file=@document.pdf" \
     -F "caption=Important document" \
     http://localhost:3000/api/chat/send-file
```

### JavaScript/Node.js Example

```javascript
const axios = require('axios');

const apiClient = axios.create({
  baseURL: 'http://localhost:3000',
  headers: {
    'Authorization': 'Bearer your-token',
    'Content-Type': 'application/json'
  }
});

// Send message
async function sendMessage(to, message) {
  try {
    const response = await apiClient.post('/api/chat/send', {
      to,
      message
    });
    console.log('Message sent:', response.data);
  } catch (error) {
    console.error('Send failed:', error.response.data);
  }
}

sendMessage('1234567890', 'Hello from Node.js!');
```

### Python Example

```python
import requests

class WhatsAppClient:
    def __init__(self, base_url, api_token):
        self.base_url = base_url
        self.headers = {
            'Authorization': f'Bearer {api_token}',
            'Content-Type': 'application/json'
        }
    
    def send_message(self, to, message):
        response = requests.post(
            f'{self.base_url}/api/chat/send',
            json={'to': to, 'message': message},
            headers=self.headers
        )
        response.raise_for_status()
        return response.json()

# Usage
client = WhatsAppClient('http://localhost:3000', 'your-token')
result = client.send_message('1234567890', 'Hello from Python!')
print(result)
```

---

## 🔗 Related Documentation

- **[Developer Guide](DEVELOPER_GUIDE.md)** - Library usage and integration
- **[Quick Reference](LIBRARY_QUICK_REFERENCE.md)** - Common patterns and examples
- **[Deployment Guide](DEPLOYMENT_GUIDE.md)** - Production deployment instructions
- **[Security Guide](SECURITY.md)** - Security best practices
