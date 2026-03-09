# 12. Appendix

## A. Glossary

| Term | Definition |
|------|------------|
| **CDP** | Chrome DevTools Protocol - API for Chrome automation |
| **Instance** | A single WhatsApp account managed by WAS |
| **MCP** | Model Context Protocol - Standard for AI tool integration |
| **QR Code** | Quick Response code used for WhatsApp authentication |
| **Pairing Code** | 8-digit alternative to QR scanning |
| **Session** | Authenticated WhatsApp connection state |
| **Webhook** | HTTP callback for real-time notifications |
| **WAL** | Write-Ahead Logging - SQLite durability mode |
| **SSE** | Server-Sent Events - Real-time data streaming |
| **JWT** | JSON Web Token - Authentication standard |

## B. API Quick Reference

### Instance Management

```bash
POST   /api/v1/instances              # Create
GET    /api/v1/instances              # List
GET    /api/v1/instances/{id}         # Get
DELETE /api/v1/instances/{id}         # Delete
```

### WhatsApp Operations

```bash
GET  /api/v1/whatsapp/{id}/qrcode       # QR code
GET  /api/v1/whatsapp/{id}/status       # Status
POST /api/v1/whatsapp/{id}/send-message # Send
GET  /api/v1/whatsapp/{id}/chats        # Chats
```

## C. Error Codes

| Code | Description |
|------|-------------|
| INSTANCE_NOT_FOUND | Instance does not exist |
| INSTANCE_NOT_CONNECTED | WhatsApp not connected |
| QR_NOT_AVAILABLE | QR code not ready |
| MESSAGE_FAILED | Message send failed |
| AUTH_REQUIRED | Authentication needed |
| RATE_LIMITED | Too many requests |

## D. Instance Status Codes

| Status | Description |
|--------|-------------|
| Initializing | Browser starting |
| QrReady | Waiting for QR scan |
| Connected | Ready for messages |
| Disconnected | Connection lost |
| Error | Instance failed |

## E. References

| Resource | URL |
|----------|-----|
| API Documentation | http://localhost:3000/api-docs/ |
| Getting Started | [GETTING_STARTED.md](../GETTING_STARTED.md) |
| Architecture | [ARCHITECTURE.md](../ARCHITECTURE.md) |
| Configuration | [CONFIGURATION.md](../CONFIGURATION.md) |
| Deployment | [DEPLOYMENT.md](../DEPLOYMENT.md) |
| MCP Specification | https://modelcontextprotocol.io |

---

[← Previous: Risks](11-risks.md) | [Back to Index](README.md)
