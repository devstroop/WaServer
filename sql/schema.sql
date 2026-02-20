-- =============================================================================
-- WAS (WhatsApp Server) Database Schema
-- =============================================================================
-- SQLite database schema for message persistence, contacts, and session data.
-- 
-- Entity Relationship:
--   messages     - Message storage (outgoing queue + incoming history)
--   conversations - Cached conversation list from WhatsApp DOM
--   contacts     - Cached contact information  
--   chat_settings - Per-chat settings (pinned, archived, muted)
--   session      - Key-value session storage
--
-- Message Model (sender/recipient):
--   - Outgoing 1:1: sender='me', recipient='contact_phone'
--   - Incoming 1:1: sender='contact_phone', recipient='me'  
--   - Outgoing group: sender='me', recipient='group_jid'
--   - Incoming group: sender='member_phone', recipient='group_jid'
--
-- =============================================================================

-- -----------------------------------------------------------------------------
-- Messages Table
-- -----------------------------------------------------------------------------
-- Unified table for both outgoing queue and incoming message history.
-- Outgoing queue: WHERE sender='me' AND status IN ('pending', 'processing')
CREATE TABLE IF NOT EXISTS messages (
    -- Primary key (UUID)
    id TEXT PRIMARY KEY,
    
    -- Sender/Recipient model
    sender TEXT NOT NULL,              -- 'me' for outgoing, phone for incoming
    recipient TEXT NOT NULL,           -- contact phone or group JID
    sender_name TEXT,                  -- Display name for incoming messages
    
    -- Content
    text TEXT,                         -- Message text or media caption
    is_group INTEGER DEFAULT 0,        -- 1 if group message
    
    -- Status tracking
    status TEXT NOT NULL,              -- pending, processing, sent, delivered, read, failed, received
    priority INTEGER DEFAULT 0,        -- Higher = processed first (for queue)
    
    -- Media attachments
    media_type TEXT NOT NULL DEFAULT 'none',  -- none, image, video, document, voice, sticker
    media_path TEXT,                   -- Local file path
    media_filename TEXT,               -- Original filename
    media_extension TEXT,              -- File extension (PDF, TOML, etc.)
    media_size INTEGER,                -- Size in bytes
    media_duration INTEGER,            -- Duration in seconds (voice/video)
    
    -- Threading
    quoted_message_id TEXT,            -- Reply-to message ID
    
    -- Error handling
    error TEXT,                        -- Error message if failed
    retry_count INTEGER DEFAULT 0,     -- Current retry attempt
    max_retries INTEGER DEFAULT 3,     -- Maximum retry attempts
    
    -- Timestamps (RFC 3339 format)
    message_timestamp TEXT,            -- WhatsApp timestamp
    created_at TEXT NOT NULL,          -- When record was created
    processed_at TEXT,                 -- When message was sent/received

    -- Legacy columns (backward compatibility)
    phone TEXT,                        -- [DEPRECATED] Use sender/recipient
    direction TEXT,                    -- [DEPRECATED] Use sender='me' check
    contact_name TEXT,                 -- [DEPRECATED] Use sender_name

    FOREIGN KEY (quoted_message_id) REFERENCES messages(id)
);

-- -----------------------------------------------------------------------------
-- Conversations Table  
-- -----------------------------------------------------------------------------
-- Cached conversation list from WhatsApp Web DOM scraping.
-- Refreshed periodically to show chat list in UI.
CREATE TABLE IF NOT EXISTS conversations (
    -- Chat identifier (phone@c.us or group ID)
    id TEXT PRIMARY KEY,
    
    -- Contact info
    phone TEXT,                        -- Phone number (if individual chat)
    name TEXT NOT NULL,                -- Contact or group name
    
    -- Preview
    last_message TEXT,                 -- Last message preview text
    last_message_time TEXT,            -- Human-readable timestamp from DOM
    unread_count INTEGER DEFAULT 0,    -- Number of unread messages
    
    -- Chat type and state
    is_group INTEGER DEFAULT 0,        -- 1 if group chat
    is_muted INTEGER DEFAULT 0,        -- 1 if notifications muted
    is_pinned INTEGER DEFAULT 0,       -- 1 if chat is pinned
    is_archived INTEGER DEFAULT 0,     -- 1 if chat is archived
    
    -- Profile
    avatar_url TEXT,                   -- Profile picture URL
    
    -- Cache metadata
    cached_at TEXT NOT NULL            -- When this was cached (RFC 3339)
);

-- -----------------------------------------------------------------------------
-- Chat Settings Table
-- -----------------------------------------------------------------------------
-- Per-chat user settings (pinned, archived, muted).
-- Persists settings even when conversation cache is refreshed.
CREATE TABLE IF NOT EXISTS chat_settings (
    -- Chat identifier
    chat_id TEXT PRIMARY KEY,
    
    -- Settings
    muted_until TEXT,                  -- Muted until timestamp (NULL = not muted)
    pinned INTEGER DEFAULT 0,          -- 1 if pinned
    archived INTEGER DEFAULT 0,        -- 1 if archived
    
    -- Metadata
    updated_at TEXT NOT NULL           -- Last update timestamp
);

-- -----------------------------------------------------------------------------
-- Contacts Table
-- -----------------------------------------------------------------------------
-- Cached contact information from WhatsApp.
CREATE TABLE IF NOT EXISTS contacts (
    -- Phone number as primary key
    phone TEXT PRIMARY KEY,
    
    -- Contact info
    name TEXT,                         -- Display name
    is_business INTEGER DEFAULT 0,     -- 1 if business account
    last_seen TEXT,                    -- Last seen timestamp (RFC 3339)
    
    -- Cache metadata
    updated_at TEXT NOT NULL           -- When record was updated
);

-- -----------------------------------------------------------------------------
-- Session Table
-- -----------------------------------------------------------------------------
-- Key-value storage for session data and settings.
CREATE TABLE IF NOT EXISTS session (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
