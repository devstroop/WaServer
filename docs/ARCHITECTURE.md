# Architecture Overview

## System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      WAS Server                          │
│  ┌───────────┐                                          │
│  │  REST API │                                          │
│  │  (Axum)   │                                          │
│  └─────┬─────┘                                          │
│        │                                                │
│        ▼                                                │
│  ┌───────────────────────────────────────────────────┐  │
│  │              Instance Manager                      │  │
│  │  ┌────────┐  ┌────────┐  ┌────────┐               │  │
│  │  │Instance│  │Instance│  │Instance│               │  │
│  │  │   #1   │  │   #2   │  │   #3   │               │  │
│  │  └───┬────┘  └───┬────┘  └───┬────┘               │  │
│  └──────┼───────────┼───────────┼────────────────────┘  │
│         └───────────┴───────────┘                       │
│                     ▼                                    │
│  ┌───────────────────────────────────────────────────┐  │
│  │            Browser Pool (Chromium)                 │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐         │  │
│  │  │ Browser  │  │ Browser  │  │ Browser  │         │  │
│  │  │ (WA Web) │  │ (WA Web) │  │ (WA Web) │         │  │
│  │  └──────────┘  └──────────┘  └──────────┘         │  │
│  └───────────────────────────────────────────────────┘  │
│  ┌───────────┐  ┌───────────┐                          │
│  │ Database  │  │  Config   │                          │
│  │ (SQLite)  │  │  (TOML)   │                          │
│  └───────────┘  └───────────┘                          │
└─────────────────────────────────────────────────────────┘
```

## Core Components

### HTTP Server (Axum)
- Async request handling with Tokio
- OpenAPI/Swagger documentation
- CORS and middleware support

### Instance Manager
- Multi-account support
- Instance lifecycle (sleep/wake)
- Request routing

### Browser Service
- Chromium automation via chromiumoxide
- Session persistence
- Dynamic element locators

## Project Structure

Layered architecture (v0.4.0) — dependencies point inward: `interfaces → application → domain`; `infrastructure` implements `application` ports.

```
src/
├── bin/was.rs               # ~116 LOC bootstrap (config, db, manager, run_server)
├── domain/                  # Pure entities — no axum/tokio/rusqlite
│   ├── instance/            # InstanceId, InstanceStatus, InstanceConfig, validation
│   ├── messaging/           # Message, MediaType, MessageStatus
│   ├── identity/            # User, UserRole, InstancePermission, TokenName/Expiry
│   └── shared/error.rs      # DomainError/DomainResult
├── application/             # Use-cases + ports (no infra deps)
│   ├── instance/            # manager.rs registry, state machine, lifecycle ports,
│   │                        #   config_validation, metadata, persistence port
│   ├── auth/                # SecretValidator, AccessToken, UserStore/TokenStore ports
│   ├── identity/            # user_service, token_service, rbac
│   └── messaging/           # SendService (validator→rate→browser), policy, ports
├── infrastructure/          # Adapters implementing application ports
│   ├── browser/             # chromiumoxide driver, session store, locators
│   ├── persistence/         # SQLite Database + user/token/instance repos
│   ├── config/              # AppConfig (TOML + env)
│   └── security/            # WhatsApp Web auth service
├── interfaces/http/         # HTTP boundary
│   ├── router.rs            # build_full_router (versioned nests + Swagger)
│   ├── middleware/stack.rs  # Trace/correlation/metrics/security/CORS/body limit
│   ├── dto/                 # Versioned ToSchema DTOs + mappers + openapi snapshot
│   └── handlers/            # Thin handlers (identity/, messaging)
├── handlers/api/            # Legacy facade handlers (instances, whatsapp, chat, users…)
├── services/whatsapp/       # InstanceService (lifecycle/auth split into sibling modules),
│                            #   InstanceManager facade, ChatService, messaging_ports adapters
├── models/                  # Compat re-exports of domain types
├── shared/observability/    # logging, service metrics, per-instance metrics
└── middleware/              # auth_middleware, correlation_id, metrics, headers
```

## Instance Lifecycle

```
SLEEPING ──request──► WARMING UP ──ready──► ACTIVE
    ▲                                          │
    └────────────idle timeout──────────────────┘
                                               │
                                        error  ▼
                                            ERROR
```

Transitions validated by `application::instance::InstanceState` (`can_transition`/`transition`).

## Request Flow

1. Request arrives at HTTP server
2. Auth middleware validates token
3. Router dispatches to handler
4. Handler gets instance from manager
5. Instance warms up if sleeping
6. Browser executes operation
7. Response returned to client

## Database Schema

```sql
CREATE TABLE instances (
    id TEXT PRIMARY KEY,
    name TEXT,
    phone_number TEXT,
    created_at TEXT,
    updated_at TEXT
);

CREATE TABLE sessions (
    instance_id TEXT PRIMARY KEY,
    data BLOB,
    updated_at TEXT
);
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| default | REST API only |
