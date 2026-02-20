# WAS - Architecture Issues & Improvements

> This document tracks architectural gaps, refactoring needs, and planned improvements.
> Items will be implemented once finalized and approved.

---

## 🔴 Critical - API Restructure

### Issue #1: Instance ID should be UUID, not phone number

**Current State:**
- Account ID = phone number (E.164 format)
- Phone required at creation time
- Can't create instance before knowing the phone

**Problem:**
- QR auth flow: User doesn't know phone until AFTER scanning QR
- Phone pairing: Same issue
- No way to create "blank" instance first

**Proposed Solution:**
```
Instance {
    id: Uuid,                        // Primary key, generated at creation
    phone_number: Option<String>,    // Set after first successful auth
    display_name: Option<String>,
    status: InstanceStatus,
    created_at: DateTime<Utc>,
    last_activity: Option<DateTime<Utc>>,
    data_dir: PathBuf,
}
```

**Phone Binding Rules:**
| State | phone_number | Action | Result |
|-------|--------------|--------|--------|
| New | `None` | Auth with any phone | Phone captured & stored |
| Bound | `Some("+1...")` | Re-auth same phone | OK |
| Bound | `Some("+1...")` | Auth different phone | **Error** |

---

### Issue #2: Rename "accounts" → "instances"

**Current State:**
- `/api/v1/admin/accounts` - WhatsApp instance management
- `/api/v1/account` - Profile/privacy operations
- "account" also means local user account (JWT auth)

**Problem:**
- Confusing terminology
- "admin" prefix suggests administration, but it's core functionality

**Proposed Solution:**
- Rename to `/api/v1/instances/:id/...` for all WhatsApp operations
- Keep `/api/v1/auth/...` for server authentication

---

### Issue #3: Remove X-Account-Id header pattern

**Current State:**
- Instance management: ID in path (`/accounts/:id`)
- WhatsApp operations: ID in header (`X-Account-Id`)

**Problem:**
- Inconsistent API design
- Non-RESTful
- Complex middleware

**Proposed Solution:**
- All operations use path parameter: `/api/v1/whatsapp/:id/chats`
- Remove `account_middleware` that extracts header
- Simpler, more intuitive API

---

### Issue #4: Unified route structure

**Current State:**
```
/api/health, /api/ready, /api/live, /api/metrics
/api/v1/admin/auth/*
/api/v1/admin/accounts/*
/api/v1/auth/*          (WhatsApp auth)
/api/v1/account/*       (profile/privacy)
/api/v1/chats/*
/api/v1/messages/*
```

**Proposed Structure:**
```
/api/v1/
├── auth/                    # Server authentication (JWT)
│   ├── POST   /login
│   ├── POST   /logout
│   ├── POST   /refresh
│   ├── GET    /status
│   └── POST   /setup
│
├── whatsapp/                # All WhatsApp instance operations
│   ├── GET    /             # List instances
│   ├── POST   /             # Create instance → {id: uuid}
│   ├── POST   /discover     # Scan filesystem
│   │
│   └── /:id/                # Instance operations
│       ├── GET    /         # Instance info
│       ├── DELETE /         # Delete instance
│       ├── POST   /start    # Start browser
│       ├── POST   /stop     # Stop browser
│       │
│       ├── GET    /session      # WA auth status
│       ├── DELETE /session      # WA logout
│       ├── GET    /link/qr      # QR code
│       ├── POST   /link/phone   # Phone pairing
│       │
│       ├── GET    /profile
│       ├── PUT    /profile
│       ├── GET    /privacy
│       ├── PUT    /privacy
│       │
│       ├── GET    /chats
│       ├── GET    /chats/:chat_id
│       ├── GET    /chats/:chat_id/events   # SSE
│       │
│       ├── POST   /messages
│       └── GET    /messages/:id
│
└── system/                  # Health/monitoring
    ├── GET    /health
    ├── GET    /ready
    ├── GET    /live
    └── GET    /metrics
```

---

## 🟡 Medium Priority

### Issue #5: Phone number verification on re-auth

**Current State:**
- `on_whatsapp_authenticated()` method exists but verification is incomplete

**Needed:**
- After QR/phone auth completes, detect the authenticated phone number
- Compare with stored `phone_number` on instance
- If mismatch and instance already bound → reject and logout
- If first auth (phone is None) → store the phone number

---

### Issue #6: MCP tools need route updates

**Current State:**
- MCP server exposes tools that call internal API routes
- Route changes will break MCP tool implementations

**After restructure:**
- Update all MCP tool implementations to use new paths
- `/api/v1/whatsapp/:id/...` pattern

---

### Issue #7: Integration tests use old routes

**File:** `tests/integration_tests.rs`

**Current State:**
- Tests reference `/api/auth/*` paths (old)
- Will fail after route restructure

**Action:**
- Update all test route references after restructure

---

## 🟢 Low Priority / Future

### Issue #8: Multi-Tenant SaaS Model (Option A: Single Deployment)

**Decision:** Multi-tenant single deployment with tenant isolation at data layer.

**Architecture:**
```
[Tenant A] ──┐
[Tenant B] ──┼─→ [WAS SaaS] → [Instances Pool]
[Tenant C] ──┘
```

**Data Model Changes:**

```rust
// Tenant (customer)
struct Tenant {
    id: Uuid,
    name: String,
    email: String,
    plan: Plan,                    // Free, Pro, Enterprise
    created_at: DateTime<Utc>,
    settings: TenantSettings,
}

// API Key (multiple per tenant)  
struct ApiKey {
    id: Uuid,
    tenant_id: Uuid,
    key_hash: String,              // Hashed, never stored plain
    name: String,                  // "Production", "Staging"
    scopes: Vec<Scope>,            // read, write, admin
    last_used: Option<DateTime<Utc>>,
    expires_at: Option<DateTime<Utc>>,
}

// Instance (owned by tenant)
struct Instance {
    id: Uuid,
    tenant_id: Uuid,               // Owner - NEW
    phone_number: Option<String>,
    display_name: Option<String>,
    status: InstanceStatus,
    created_at: DateTime<Utc>,
}

// Usage tracking
struct UsageRecord {
    tenant_id: Uuid,
    instance_id: Uuid,
    period: String,                // "2026-02"
    messages_sent: u64,
    messages_received: u64,
    media_bytes: u64,
}
```

**API Routes for SaaS:**

```
# Public (no auth)
POST   /api/v1/auth/login
POST   /api/v1/auth/refresh
POST   /api/v1/auth/register       # Self-service signup

# Tenant-scoped (automatic filtering by JWT tenant_id)
GET    /api/v1/whatsapp/           # Only YOUR instances
POST   /api/v1/whatsapp/           # Creates under YOUR tenant
...all instance routes...

# Usage & billing (tenant-scoped)
GET    /api/v1/usage               # Your current period
GET    /api/v1/usage/history       # Historical

# API key management (tenant-scoped)
GET    /api/v1/keys                # List your API keys
POST   /api/v1/keys                # Create new key
DELETE /api/v1/keys/:id            # Revoke key

# Platform admin only
GET    /api/v1/admin/tenants
POST   /api/v1/admin/tenants
GET    /api/v1/admin/tenants/:id
DELETE /api/v1/admin/tenants/:id
GET    /api/v1/admin/usage/all     # All tenants usage
```

**Middleware Changes:**
```rust
// Extract tenant_id from JWT, auto-filter all queries
let tenant_id = jwt_claims.tenant_id;
let instances = manager.list_instances_for_tenant(tenant_id).await;
```

**SaaS Feature Priorities:**

| Priority | Feature | Notes |
|----------|---------|-------|
| P0 | Tenant isolation | Data, instances, logs separated |
| P0 | Usage metering | Messages per tenant |
| P0 | Rate limiting | Per-tenant quotas |
| P1 | Instance limits | Max per plan |
| P1 | API key management | Multiple keys, scopes |
| P1 | Audit logging | Who did what |
| P2 | Billing webhooks | Usage threshold alerts |
| P2 | Tenant webhooks | Per-tenant callback URLs |
| P3 | Admin dashboard | Tenant management UI |
| P3 | Self-service signup | Registration flow |

**Infrastructure Requirements:**

| Component | Current | SaaS |
|-----------|---------|------|
| Database | SQLite | **PostgreSQL** |
| Storage | Local | **S3/GCS** for media |
| Auth | Local JWT | Keep or **Auth0/Clerk** |
| Billing | N/A | **Stripe** |
| Monitoring | Optional | **Required** |

---

### Issue #9: Database for instance metadata

**Current State:**
- Instance info derived from filesystem (data directories)
- No persistent metadata store

**Consideration:**
- SQLite for instance metadata (id, phone, created_at, etc.)
- Filesystem for browser profiles only
- Enables richer queries, history, audit logs

---

## 📝 Implementation Order

### Phase 1: API Restructure (Current Focus)

1. **Models** - Update `Instance` struct with UUID id, optional phone
2. **Storage** - Update filesystem handling for UUID-based directories
3. **Routes** - Restructure to `/api/v1/whatsapp/:id/` pattern
4. **Handlers** - Remove X-Account-Id, use path params
5. **Middleware** - Simplify (remove account_middleware)
6. **Phone binding** - Implement verification on auth completion
7. **OpenAPI** - Update all utoipa annotations
8. **MCP** - Update tool implementations
9. **Tests** - Update integration tests
10. **Docs** - Update README, API documentation

### Phase 2: SaaS Foundation

1. **Database** - Add PostgreSQL support (SQLx)
2. **Tenant model** - Create Tenant, ApiKey tables
3. **Auth upgrade** - Add tenant_id to JWT claims
4. **Middleware** - Auto-filter by tenant
5. **Migration** - SQLite → PostgreSQL migration path

### Phase 3: SaaS Features

1. **Usage tracking** - Message counting per tenant
2. **Rate limiting** - Per-tenant quotas
3. **API keys** - Multi-key support with scopes
4. **Instance limits** - Plan-based limits
5. **Audit logging** - Activity tracking

### Phase 4: Billing & Growth

1. **Stripe integration** - Plans, subscriptions
2. **Usage webhooks** - Threshold alerts
3. **Self-service signup** - Registration flow
4. **Admin dashboard** - Tenant management

### Phase 5: Horizontal Scaling & Clustering

**Architecture:**
```
                    ┌─────────────────┐
                    │  Load Balancer  │
                    └────────┬────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
        ▼                    ▼                    ▼
┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│   API Node 1  │   │   API Node 2  │   │   API Node N  │
│  (Stateless)  │   │  (Stateless)  │   │  (Stateless)  │
└───────┬───────┘   └───────┬───────┘   └───────┬───────┘
        │                   │                   │
        └───────────────────┼───────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
        ▼                   ▼                   ▼
┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│   Worker 1    │   │   Worker 2    │   │   Worker N    │
│  (Browser)    │   │  (Browser)    │   │  (Browser)    │
└───────────────┘   └───────────────┘   └───────────────┘
        │                   │                   │
        └───────────────────┼───────────────────┘
                            │
        ┌───────────────────┴───────────────────┐
        │                                       │
        ▼                                       ▼
┌───────────────────────┐         ┌───────────────────────┐
│     PostgreSQL        │         │    Redis Cluster      │
│  (Primary + Replicas) │         │  (Sessions + Queue)   │
└───────────────────────┘         └───────────────────────┘
```

**Components:**

| Component | Type | Role |
|-----------|------|------|
| API Nodes | Stateless | HTTP handlers, routing, auth |
| Worker Nodes | Stateful | Browser instances, WebSocket |
| PostgreSQL | Shared | Tenant data, instance metadata |
| Redis | Shared | Session state, job queue, pub/sub |
| S3/GCS | Shared | Media storage |

**Key Features:**

1. **Instance Routing** - Route requests to correct worker via Redis registry
2. **Worker Registry** - Track which worker owns which instance
3. **Health Monitoring** - Automatic failover on worker death
4. **Auto-scaling** - Scale workers based on instance count
5. **Graceful Shutdown** - Migrate instances before termination

**Kubernetes Deployment:**
```yaml
# API: Deployment (stateless, scale freely)
replicas: 3
strategy: RollingUpdate

# Workers: StatefulSet (stable network identity)
replicas: 5
podManagementPolicy: Parallel
```

**Implementation:**

1. **Redis integration** - Session store, pub/sub for events
2. **Worker discovery** - Service mesh or Redis-based registry
3. **Instance affinity** - Sticky routing to worker node
4. **Queue system** - BullMQ or custom Redis-based
5. **Health checks** - Kubernetes probes, self-healing
6. **Metrics** - Prometheus integration per worker

### Phase 6: Security & Compliance

**Data Protection:**

| Layer | Requirement | Implementation |
|-------|-------------|----------------|
| At Rest | Encryption | PostgreSQL TDE, S3 SSE |
| In Transit | TLS 1.3 | Nginx/Envoy termination |
| Secrets | Vault | HashiCorp Vault / K8s Secrets |
| PII | Masking | Audit logs, support access |

**Access Control:**

1. **RBAC** - Role-based permissions per tenant
2. **Scopes** - Fine-grained API key permissions
3. **IP Allowlist** - Optional per-tenant restriction
4. **MFA** - Admin dashboard requirement

**Audit & Compliance:**

- [ ] Audit logging - All API actions with actor
- [ ] Data export - GDPR Article 15 compliance
- [ ] Data deletion - GDPR Article 17 (right to erasure)
- [ ] Data residency - Region-specific deployments
- [ ] Retention policies - Configurable per tenant
- [ ] Penetration testing - Annual third-party audit
- [ ] SOC 2 Type II - Trust services criteria
- [ ] ISO 27001 - Information security management

**WhatsApp-Specific:**

| Area | Requirement |
|------|-------------|
| TOS | Comply with WhatsApp Terms of Service |
| Anti-Spam | Rate limiting, content filtering |
| Ban Recovery | Instance isolation, not platform-wide |
| Business API | Consider official API for enterprise tier |

**Security Hardening:**

1. **Container security** - Non-root, read-only FS, seccomp
2. **Network policies** - Kubernetes NetworkPolicy isolation
3. **Secrets rotation** - Automated key rotation
4. **Dependency scanning** - cargo-audit, Snyk
5. **SAST/DAST** - Static and dynamic analysis in CI

---

*Last updated: 2026-02-20*
