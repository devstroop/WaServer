# WAS Database Schema

SQLite database schema for WhatsApp Server message persistence and user access control.

## Files

| File | Description |
|------|-------------|
| [schema.sql](schema.sql) | Message table definitions |
| [users.sql](users.sql) | User and RBAC table definitions |
| [indexes.sql](indexes.sql) | Performance indexes |
| [migrations.sql](migrations.sql) | Schema migration scripts |
| [ERD.sql](ERD.sql) | Entity relationship diagram (ASCII) |

## Architecture

WAS uses two database locations:

| Database | Location | Purpose |
|----------|----------|---------|
| Central | `~/.was/data/database.db` | Users, roles, instance ownership |
| Per-instance | `~/.was/accounts/<phone>/database.db` | Messages, contacts, conversations |

## Tables - Central Database (RBAC)

```
┌──────────────────┐           ┌──────────────────┐
│      users       │──────────<│ instance_access  │
├──────────────────┤           ├──────────────────┤
│ • Authentication │           │ • Shared access  │
│ • Admin flag     │           │ • Permissions    │
│ • Active status  │           │ • Expiry         │
└────────┬─────────┘           └──────────────────┘
         │owns                         │
         │                             │
         ▼                             ▼
┌──────────────────┐           ┌──────────────────┐
│    instances     │           │ refresh_tokens   │
├──────────────────┤           ├──────────────────┤
│ • Instance meta  │           │ • Token tracking │
│ • Owner ID       │           │ • Revocation     │
│ • Display name   │           │ • Per-device     │
└──────────────────┘           └──────────────────┘
```

## Tables - Per-Instance Database

```
┌──────────────────┐     ┌──────────────────┐     ┌──────────────────┐
│     messages     │     │  conversations   │     │     contacts     │
├──────────────────┤     ├──────────────────┤     ├──────────────────┤
│ • Outgoing queue │     │ • Chat list      │     │ • Contact cache  │
│ • Message history│     │ • DOM scraped    │     │ • Name, business │
│ • Media support  │     │ • Unread counts  │     │ • Last seen      │
└──────────────────┘     └──────────────────┘     └──────────────────┘

┌──────────────────┐     ┌──────────────────┐
│  chat_settings   │     │     session      │
├──────────────────┤     ├──────────────────┤
│ • Pinned chats   │     │ • Key-value store│
│ • Muted chats    │     │ • Login tracking │
│ • Archived chats │     │                  │
└──────────────────┘     └──────────────────┘
```

## Message Model

Uses standard sender/recipient model (like email):

| Scenario | sender | recipient |
|----------|--------|-----------|
| Outgoing 1:1 | `'me'` | `contact_phone` |
| Incoming 1:1 | `contact_phone` | `'me'` |
| Outgoing group | `'me'` | `group_jid` |
| Incoming group | `member_phone` | `group_jid` |

## Message Status Flow

```
OUTGOING:  pending → processing → sent → delivered → read
              │                    │
              └────► failed ◄──────┘
                    (retry)

INCOMING:  received
```

## Quick Queries

```sql
-- Get outgoing queue
SELECT * FROM messages 
WHERE sender = 'me' AND status IN ('pending', 'processing')
ORDER BY priority DESC, created_at ASC;

-- Get chat history with a contact
SELECT * FROM messages 
WHERE recipient = ? OR (sender = ? AND recipient = 'me')
ORDER BY created_at DESC
LIMIT 50;

-- Get unread count
SELECT COUNT(*) FROM messages 
WHERE sender != 'me' AND status = 'received';

-- Get queue status
SELECT 
    SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END) as pending,
    SUM(CASE WHEN status = 'processing' THEN 1 ELSE 0 END) as processing,
    SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) as failed
FROM messages WHERE sender = 'me';
```

## Usage

The schema is auto-created by `DatabaseService` on startup. These SQL files are for documentation and manual operations only.

```rust
// Schema is initialized automatically
let db = DatabaseService::new("./data")?;
```

## RBAC Permissions

Instance access is determined by:

1. **Ownership**: Creator of the instance has full control
2. **Admin flag**: Users with `is_admin=1` have full access to all instances
3. **Shared access**: Via `instance_access` table with granular permissions

| Permission | Owner | Admin | Shared |
|------------|-------|-------|--------|
| Read | ✓ | ✓ | Optional |
| Send | ✓ | ✓ | Optional |
| Manage | ✓ | ✓ | Optional |
| Delete | ✓ | ✓ | ✗ |
| Share | ✓ | ✓ | ✗ |

## Quick Queries (RBAC)

```sql
-- Get all instances accessible by a user
SELECT i.*, 
    CASE 
        WHEN i.owner_id = :user_id THEN 'owner'
        WHEN u.is_admin = 1 THEN 'admin'
        ELSE 'shared'
    END as access_type
FROM instances i
LEFT JOIN users u ON u.id = :user_id
LEFT JOIN instance_access a ON a.instance_id = i.id AND a.user_id = :user_id
WHERE i.owner_id = :user_id 
   OR u.is_admin = 1
   OR (a.user_id IS NOT NULL AND (a.expires_at IS NULL OR a.expires_at > datetime('now')));

-- Check user's permission on an instance
SELECT 
    CASE WHEN i.owner_id = :user_id THEN 1 ELSE 0 END as is_owner,
    COALESCE(a.can_read, 0) as can_read,
    COALESCE(a.can_send, 0) as can_send,
    COALESCE(a.can_manage, 0) as can_manage
FROM instances i
LEFT JOIN instance_access a ON a.instance_id = i.id AND a.user_id = :user_id
WHERE i.id = :instance_id;

-- List users with access to an instance
SELECT u.id, u.username, u.display_name,
    CASE WHEN i.owner_id = u.id THEN 'owner' ELSE 'shared' END as access_type,
    a.can_read, a.can_send, a.can_manage, a.expires_at
FROM users u
LEFT JOIN instances i ON i.id = :instance_id
LEFT JOIN instance_access a ON a.instance_id = i.id AND a.user_id = u.id
WHERE i.owner_id = u.id OR a.user_id = u.id;
```
