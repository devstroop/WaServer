-- =============================================================================
-- Users & Instance Ownership Schema
-- =============================================================================
-- User authentication, instance management, and ownership system.
--
-- Design Principles:
--   - Users are the primary actors in the system
--   - Instances are owned by users (one owner per instance)
--   - Instance access can be shared with other users
--   - Future: Roles and permissions will be added for fine-grained access control
--
-- Ownership Model:
--   - Owner: User who created the instance (full control)
--   - Shared: Users granted access via instance_access table
--   - Admin: System users with global access (future)
--
-- =============================================================================

-- -----------------------------------------------------------------------------
-- Users Table
-- -----------------------------------------------------------------------------
-- User accounts for authentication and ownership tracking.
-- Replaces the in-memory user storage in AuthTokenService.
CREATE TABLE IF NOT EXISTS users (
    -- Primary key (UUID)
    id TEXT PRIMARY KEY,
    
    -- Credentials
    username TEXT NOT NULL UNIQUE,     -- Login username (case-insensitive lookup)
    password_hash TEXT NOT NULL,       -- bcrypt hashed password
    
    -- Profile
    email TEXT,                        -- Optional email for notifications
    display_name TEXT,                 -- Display name (defaults to username)
    
    -- Status
    is_active INTEGER DEFAULT 1,       -- 1 if account is active
    is_admin INTEGER DEFAULT 0,        -- 1 if system administrator
    
    -- Metadata
    created_at TEXT NOT NULL,          -- When account was created (RFC 3339)
    updated_at TEXT NOT NULL,          -- Last profile update (RFC 3339)
    last_login_at TEXT                 -- Last successful login (RFC 3339)
);

-- -----------------------------------------------------------------------------
-- Instances Table
-- -----------------------------------------------------------------------------
-- WhatsApp instance metadata and ownership.
-- Each instance represents a WhatsApp Web session with isolated resources.
CREATE TABLE IF NOT EXISTS instances (
    -- Primary key (instance name acts as ID)
    id TEXT PRIMARY KEY,
    
    -- Ownership
    owner_id TEXT NOT NULL,            -- User who owns this instance
    
    -- Display info
    display_name TEXT,                 -- Human-friendly name
    description TEXT,                  -- Optional description
    
    -- Status
    is_active INTEGER DEFAULT 1,       -- 1 if instance is active
    
    -- Metadata
    created_at TEXT NOT NULL,          -- When instance was created (RFC 3339)
    updated_at TEXT NOT NULL,          -- Last update (RFC 3339)
    
    FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE
);

-- -----------------------------------------------------------------------------
-- Instance Access Table
-- -----------------------------------------------------------------------------
-- Shared access to instances for users other than the owner.
-- Allows instance owners to grant specific permissions to other users.
CREATE TABLE IF NOT EXISTS instance_access (
    -- Composite primary key
    instance_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    
    -- Permissions (future: will migrate to permission bits)
    can_read INTEGER DEFAULT 1,        -- Can view chats and messages
    can_send INTEGER DEFAULT 0,        -- Can send messages
    can_manage INTEGER DEFAULT 0,      -- Can modify instance settings
    
    -- Grant metadata
    granted_by TEXT NOT NULL,          -- User who granted access
    granted_at TEXT NOT NULL,          -- When access was granted (RFC 3339)
    expires_at TEXT,                   -- Optional expiry (RFC 3339, NULL = never)
    
    PRIMARY KEY (instance_id, user_id),
    FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (granted_by) REFERENCES users(id)
);

-- -----------------------------------------------------------------------------
-- Refresh Tokens Table
-- -----------------------------------------------------------------------------
-- Track issued refresh tokens for logout/revocation per device.
-- Replaces the in-memory revoked_tokens list.
CREATE TABLE IF NOT EXISTS refresh_tokens (
    -- Token identifier (hash of the token, not the token itself)
    id TEXT PRIMARY KEY,
    
    -- Ownership
    user_id TEXT NOT NULL,             -- User this token belongs to
    
    -- Token metadata
    issued_at TEXT NOT NULL,           -- When token was issued (RFC 3339)
    expires_at TEXT NOT NULL,          -- When token expires (RFC 3339)
    
    -- Device info (optional)
    device_name TEXT,                  -- User-friendly device name
    device_fingerprint TEXT,           -- Unique device identifier
    
    -- Revocation
    revoked_at TEXT,                   -- When token was revoked (NULL = active)
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- -----------------------------------------------------------------------------
-- Password Reset Tokens Table
-- -----------------------------------------------------------------------------
-- Temporary tokens for password reset flow.
-- Replaces the in-memory reset_tokens list.
CREATE TABLE IF NOT EXISTS password_reset_tokens (
    -- Token value (UUID)
    token TEXT PRIMARY KEY,
    
    -- Target user
    user_id TEXT NOT NULL,
    
    -- Expiry
    expires_at TEXT NOT NULL,          -- When token expires (RFC 3339)
    
    -- Usage tracking
    used_at TEXT,                      -- When token was used (NULL = unused)
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
