# ISSUES - Migration Plan: React to HTMX

## Overview

Migrate the existing React frontend (`app/`) to a server-rendered HTMX approach for simpler architecture and reduced complexity.

---

## Phase 1: Setup & Infrastructure

- [x] **1.1** Add Askama templating crate to Cargo.toml
- [x] **1.2** Create `templates/` directory structure
- [x] **1.3** Set up base HTML layout with HTMX, Tailwind CSS CDN
- [x] **1.4** Configure static file serving for assets

## Phase 2: Core Pages

- [x] **2.1** Dashboard page - server stats, connection status, uptime
- [x] **2.2** Auth page - QR code display with auto-refresh via HTMX polling
- [x] **2.3** Chat page - message list with SSE streaming updates
- [x] **2.4** Settings page - forms with HTMX submissions

## Phase 3: Components & Interactions

- [x] **3.1** Navigation/sidebar component
- [x] **3.2** Toast notifications via HTMX OOB swaps
- [x] **3.3** Modal dialogs for confirmations
- [x] **3.4** Dark/light theme toggle (localStorage + CSS variables)

## Phase 4: API Token & Webhook Management

- [x] **4.1** Access tokens page - CRUD with inline editing
- [x] **4.2** Webhooks page - endpoint management forms

## Phase 5: Cleanup & Polish

- [x] **5.1** Remove React app directory (`app/`)
- [x] **5.2** Update README.md with new build instructions
- [x] **5.3** Update Docker configuration (no changes needed - templates built-in)
- [x] **5.4** Test all flows end-to-end

---

## Technical Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Templating | Askama | Compile-time checked, fast, Rust-native |
| CSS | Tailwind CDN | No build step, familiar from React version |
| JS Framework | HTMX + Alpine.js | HTMX for server interactions, Alpine for client-side state |
| Icons | Lucide (CDN) | Same icons as React version |

## File Structure (Target)

```
templates/
├── base.html           # Base layout with head, scripts, nav
├── pages/
│   ├── dashboard.html
│   ├── auth.html
│   ├── chat.html
│   ├── settings.html
│   ├── tokens.html
│   └── webhooks.html
├── components/
│   ├── nav.html
│   ├── toast.html
│   ├── chat_message.html
│   └── qr_code.html
└── partials/
    ├── chat_list.html
    ├── message_list.html
    └── token_row.html
```

## Dependencies to Add

```toml
# Cargo.toml additions
askama = "0.12"
askama_axum = "0.4"
```

## Notes

- HTMX `hx-trigger="sse:message"` for real-time chat updates
- Use `hx-swap-oob="true"` for updating multiple elements (e.g., toast + content)
- QR code refresh: `hx-trigger="every 5s"` until authenticated
- Alpine.js for theme toggle, dropdown menus, local UI state
