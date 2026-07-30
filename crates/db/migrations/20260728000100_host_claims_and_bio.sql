-- Host claims + host bio.
--
-- Two gaps this closes:
--   1. Seeded Spaces have host_user_id NULL and there was no way to claim them.
--      A user applies to become the host; an admin approves, which assigns
--      host_user_id. Popular places can have several applicants, so claims live
--      in their own table rather than a single column on spaces.
--   2. The host card had nothing editable. host_bio_zh/en lets a claimed host
--      write a few words that render on the Space's host panel.

ALTER TABLE spaces ADD COLUMN IF NOT EXISTS host_bio_zh text;
ALTER TABLE spaces ADD COLUMN IF NOT EXISTS host_bio_en text;

CREATE TABLE IF NOT EXISTS space_host_claims (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  space_id uuid NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
  user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  message text,
  status text NOT NULL DEFAULT 'pending',
  created_at timestamptz NOT NULL DEFAULT now(),
  decided_at timestamptz,
  UNIQUE (space_id, user_id)
);

-- Fast lookup of the review queue (pending claims, oldest first).
CREATE INDEX IF NOT EXISTS idx_space_host_claims_pending
  ON space_host_claims (created_at)
  WHERE status = 'pending';
