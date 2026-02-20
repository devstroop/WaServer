-- =============================================================================
-- WAS Database Migrations
-- =============================================================================
-- Schema migrations for existing databases.
-- These are applied automatically on startup if columns are missing.
-- =============================================================================

-- -----------------------------------------------------------------------------
-- Migration: Add sender/recipient model columns
-- -----------------------------------------------------------------------------
-- Previous schema used `phone` and `direction` columns.
-- New schema uses `sender` and `recipient` for standard messaging model.

-- Add sender column (defaults to existing contact for migration)
ALTER TABLE messages ADD COLUMN sender TEXT;

-- Add recipient column  
ALTER TABLE messages ADD COLUMN recipient TEXT;

-- Add sender_name column (for incoming message display)
ALTER TABLE messages ADD COLUMN sender_name TEXT;

-- Add is_group flag
ALTER TABLE messages ADD COLUMN is_group INTEGER DEFAULT 0;

-- -----------------------------------------------------------------------------
-- Migration: Add queue management columns
-- -----------------------------------------------------------------------------

-- Priority for send queue (higher = first)
ALTER TABLE messages ADD COLUMN priority INTEGER DEFAULT 0;

-- Maximum retry attempts before permanent failure
ALTER TABLE messages ADD COLUMN max_retries INTEGER DEFAULT 3;

-- -----------------------------------------------------------------------------
-- Data Migration: phone/direction → sender/recipient
-- -----------------------------------------------------------------------------
-- Converts old schema data to new sender/recipient model.
-- Run after column migrations are complete.

-- Migrate outgoing messages (direction='outgoing')
-- sender='me', recipient=<phone>
UPDATE messages SET 
    sender = 'me',
    recipient = phone,
    is_group = 0
WHERE (sender IS NULL OR sender = '') 
  AND direction = 'outgoing' 
  AND phone IS NOT NULL;

-- Migrate incoming messages (direction='incoming')
-- sender=<phone>, recipient='me'
UPDATE messages SET 
    sender = phone,
    recipient = 'me',
    sender_name = contact_name,
    is_group = 0
WHERE (sender IS NULL OR sender = '') 
  AND direction = 'incoming' 
  AND phone IS NOT NULL;
