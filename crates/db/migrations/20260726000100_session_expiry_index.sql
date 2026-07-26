-- Expired sessions were never pruned and expires_at had no index, so both the
-- login INSERT and the session lookup degraded as the table grew.
CREATE INDEX IF NOT EXISTS sessions_expires_at_idx ON sessions (expires_at);
CREATE INDEX IF NOT EXISTS access_sessions_expires_at_idx ON access_sessions (expires_at);

DELETE FROM sessions WHERE expires_at < now();
DELETE FROM access_sessions WHERE expires_at < now();
