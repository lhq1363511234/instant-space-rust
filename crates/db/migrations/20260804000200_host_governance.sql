-- Phase 9.3: durable Space host governance.
--
-- A Space outlives any individual host. These fields and immutable events make
-- vacancy, system care, co-hosting and handover explicit without deleting the
-- history accumulated under previous hosts.

ALTER TABLE spaces
  ADD COLUMN host_governance_state text NOT NULL DEFAULT 'hosted'
    CHECK (host_governance_state IN ('hosted', 'recruiting', 'system_care')),
  ADD COLUMN host_recruitment_note text;

UPDATE spaces
SET host_governance_state = CASE
  WHEN host_user_id IS NULL THEN 'recruiting'
  ELSE 'hosted'
END;

CREATE UNIQUE INDEX space_host_one_active_user_idx
  ON space_host_tenures (space_id, user_id)
  WHERE status = 'active';

CREATE INDEX space_host_tenures_space_history_idx
  ON space_host_tenures (space_id, started_at DESC);

CREATE TABLE space_governance_events (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  space_id uuid NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
  actor_id uuid REFERENCES users(id) ON DELETE SET NULL,
  action text NOT NULL CHECK (action IN (
    'appoint_co_host', 'appoint_steward', 'remove_host', 'leave_host',
    'transfer_primary', 'release_to_recruiting', 'place_in_system_care',
    'resume_hosted', 'update_recruitment_note'
  )),
  from_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
  to_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
  note text,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX space_governance_events_space_idx
  ON space_governance_events (space_id, created_at DESC);
CREATE INDEX space_governance_events_actor_idx
  ON space_governance_events (actor_id, created_at DESC);
