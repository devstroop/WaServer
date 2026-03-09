# 6. Use Cases

## UC-01: Send Order Confirmation

**Actor:** E-commerce Platform  
**Trigger:** Customer completes purchase

```bash
curl -X POST http://localhost:3000/api/v1/whatsapp/shop/send-message \
  -H "Authorization: Bearer token" \
  -d '{"recipient": "+1234567890", "message": "Order #12345 confirmed!"}'
```

## UC-02: AI Customer Support

**Actor:** AI Agent (Claude via MCP)  
**Trigger:** AI decides to send message

```json
{
  "tool": "send_message",
  "arguments": {
    "instance_id": "support-bot",
    "recipient": "+1234567890",
    "message": "Your refund has been processed."
  }
}
```

## UC-03: Appointment Reminder

**Actor:** Healthcare System  
**Trigger:** Scheduled job 24 hours before appointment

## UC-04: Two-Way Support Chat

**Actor:** Customer Support Agent  
**Trigger:** Customer sends message (webhook)

## UC-05: Lead Notification

**Actor:** CRM System  
**Trigger:** New lead submitted

## UC-06: Marketing Campaign

**Actor:** Marketing Platform  
**Trigger:** Campaign scheduled send

---

[← Previous: Features](05-features.md) | [Next: Technical Requirements →](07-technical-requirements.md)
