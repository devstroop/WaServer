# 8. Non-Functional Requirements

## Performance Requirements

### NFR-P1: API Response Time

| Metric | Target | Maximum |
|--------|--------|---------|
| Health check | < 10ms | 50ms |
| List instances | < 50ms | 200ms |
| Send message | < 200ms | 1s |
| Get QR code | < 500ms | 2s |

### NFR-P2: Resource Efficiency

| Resource | Idle | Active |
|----------|------|--------|
| Memory per instance | 50 MB | 200 MB |
| CPU per instance | < 1% | < 10% |

## Reliability Requirements

### NFR-R1: Availability

| SLA Level | Uptime | Downtime/Month |
|-----------|--------|----------------|
| Target | 99.9% | 43 minutes |
| Minimum | 99.5% | 3.6 hours |

### NFR-R2: Fault Tolerance

| Failure | Recovery |
|---------|----------|
| Browser crash | Auto-restart < 30s |
| Network disconnect | Auto-reconnect |
| Server crash | Resume from DB state |

## Security Requirements

### NFR-SEC1: Authentication

| Mechanism | Purpose |
|-----------|---------|
| API Key | Programmatic access |
| JWT Token | Session auth |
| Instance Token | Per-instance access |

### NFR-SEC2: Data Protection

| Data | At Rest | In Transit |
|------|---------|------------|
| Credentials | Encrypted | TLS 1.2+ |
| Sessions | File permissions | HTTPS |
| API Keys | Hashed | Never logged |

## Scalability Requirements

### NFR-S1: Vertical Scaling

| Server Size | Instances | RAM |
|-------------|-----------|-----|
| Small | 10 | 2 GB |
| Medium | 50 | 8 GB |
| Large | 100 | 16 GB |

---

[← Previous: Technical Requirements](07-technical-requirements.md) | [Next: Success Metrics →](09-success-metrics.md)
