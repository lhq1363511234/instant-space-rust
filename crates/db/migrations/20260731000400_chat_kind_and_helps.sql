-- Phase 6: chat message kinds + helps minimal closed loop.
-- `kind` distinguishes user text from system/hosted events (help raised,
-- help resolved, host messages) so the UI can render them differently.
ALTER TABLE chat_messages ADD COLUMN IF NOT EXISTS kind text NOT NULL DEFAULT 'text';

CREATE INDEX IF NOT EXISTS idx_chat_messages_room_time
  ON chat_messages (space_id, created_at);

-- Helps: a space visitor can raise a help request; the host resolves it.
-- `resolved_at` marks the closed loop and hides it from the active list.
ALTER TABLE helps ADD COLUMN IF NOT EXISTS requester_name text;
ALTER TABLE helps ADD COLUMN IF NOT EXISTS resolved_at timestamptz;
