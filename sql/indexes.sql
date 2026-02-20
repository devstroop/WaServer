-- =============================================================================
-- WAS Database Indexes
-- =============================================================================
-- Performance indexes for common query patterns.
-- =============================================================================

-- -----------------------------------------------------------------------------
-- Message Queue Indexes
-- -----------------------------------------------------------------------------

-- Queue processing: get next pending message by priority
-- Used by: dequeue_next(), get_next_pending()
CREATE INDEX IF NOT EXISTS idx_messages_queue 
ON messages(sender, status, priority DESC, created_at ASC)
WHERE sender = 'me' AND status IN ('pending', 'processing');

-- Priority-based ordering for queue
CREATE INDEX IF NOT EXISTS idx_messages_priority 
ON messages(priority DESC, created_at ASC);

-- -----------------------------------------------------------------------------
-- Message History Indexes
-- -----------------------------------------------------------------------------

-- Chat history lookup: messages for a specific chat/recipient
-- Used by: get_messages_for_chat()
CREATE INDEX IF NOT EXISTS idx_messages_chat 
ON messages(recipient, created_at DESC);

-- Status filtering (pending, sent, failed, etc.)
-- Used by: get_pending_count(), queue status queries
CREATE INDEX IF NOT EXISTS idx_messages_status 
ON messages(status);

-- Sender lookup (for filtering outgoing vs incoming)
CREATE INDEX IF NOT EXISTS idx_messages_sender 
ON messages(sender);

-- Recipient lookup
CREATE INDEX IF NOT EXISTS idx_messages_recipient 
ON messages(recipient);

-- Chat lookup: combined sender/recipient for conversation view
-- Used by: get_conversations()
CREATE INDEX IF NOT EXISTS idx_messages_chat_lookup 
ON messages(sender, recipient, created_at DESC);

-- Media type filtering
CREATE INDEX IF NOT EXISTS idx_messages_media_type 
ON messages(media_type);

-- Time-based queries (retention policy, date filtering)
CREATE INDEX IF NOT EXISTS idx_messages_created_at 
ON messages(created_at);

-- -----------------------------------------------------------------------------
-- Conversation Cache Indexes
-- -----------------------------------------------------------------------------

-- Cache freshness check
-- Used by: is_conversation_cache_stale()
CREATE INDEX IF NOT EXISTS idx_conversations_cached 
ON conversations(cached_at);
