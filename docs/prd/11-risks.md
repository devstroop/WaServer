# 11. Risks & Mitigations

## Risk Overview

| Category | Risk Level | Count |
|----------|------------|-------|
| Technical | High | 4 |
| Business | Medium | 3 |
| Legal | High | 2 |

## Technical Risks

### TR-01: WhatsApp Web Changes

**Risk Level:** HIGH

WhatsApp regularly updates their web interface, which may break automation.

**Mitigation:**
- Configurable locators in TOML (✅ Implemented)
- Fallback selectors (✅ Implemented)
- Rapid response updates

### TR-02: WhatsApp Rate Limiting

**Risk Level:** MEDIUM-HIGH

WhatsApp may block accounts that send too many messages.

**Mitigation:**
- Built-in rate limiting (Planned)
- Ban detection (✅ Implemented)
- User education on best practices

### TR-03: Chrome Compatibility

**Risk Level:** MEDIUM

Chrome updates may break CDP automation.

**Mitigation:**
- Version pinning in Docker (✅ Implemented)
- chromiumoxide updates monitoring

### TR-04: Session Persistence Failures

**Risk Level:** MEDIUM

Sessions may become invalid requiring re-authentication.

**Mitigation:**
- SQLite WAL mode (✅ Implemented)
- Session validation on start (✅ Implemented)

## Business Risks

### BR-01: Limited Market Adoption

**Mitigation:** Unique value prop (MCP, multi-instance), documentation, community

### BR-02: Open Source Sustainability

**Mitigation:** GitHub Sponsors, Enterprise support tier, consulting services

### BR-03: Competition from Official API

**Mitigation:** Lower barrier (no approval), no per-message fees

## Legal Risks

### LR-01: WhatsApp Terms of Service

**Risk Level:** HIGH

**Legal Position:** WAS is a tool; users are responsible for compliance.

### LR-02: Data Protection (GDPR/CCPA)

**Risk Level:** MEDIUM

**Mitigation:** Self-hosted, no central data collection

---

[← Previous: Roadmap](10-roadmap.md) | [Next: Appendix →](12-appendix.md)
