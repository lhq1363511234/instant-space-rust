-- A cloud home is not a parallel object beside Space. It is the user's own
-- private Space, with the life-specific home record layered on top.
--
-- Legacy spaces require coordinates. Home Spaces therefore use the neutral
-- 0/0 sentinel internally, are marked category=cloud_home, remain private,
-- and are hard-excluded from public map/discovery queries. No real household
-- address is collected or exposed.

ALTER TABLE cloud_homes
  ADD COLUMN space_id uuid UNIQUE REFERENCES spaces(id) ON DELETE CASCADE;

WITH created_home_spaces AS (
  INSERT INTO spaces (
    name_zh, name_en, space_type, custom_type, category,
    lat, lng, is_public, password_hash, duration_hours, expires_at,
    status, creator_id, host_user_id, description_zh, description_en,
    tag_zh, tag_en
  )
  SELECT
    h.name,
    'Cloud Home',
    'custom'::space_type,
    '家空间',
    'cloud_home',
    0,
    0,
    false,
    COALESCE(NULLIF(h.passphrase_hash, ''), '$argon2id$v=19$m=19456,t=2,p=1$L/eaj4KH/9vcKXJqKl+nvg$wb5DLaTJ/X77Oqutf5gYTikcc8Zox+wCa0WW/JGHtHU'),
    0,
    NULL,
    'active'::space_status,
    h.owner_id,
    h.owner_id,
    '这是用户在 inspace 的家。离线时，家人在此歇息；出门时，足迹随行。',
    'The user''s home in inspace. Companions rest here while away and travel with the user when present.',
    '云上家',
    'Cloud Home'
  FROM cloud_homes h
  WHERE h.space_id IS NULL
  RETURNING id, creator_id
)
UPDATE cloud_homes h
SET space_id = s.id,
    updated_at = now()
FROM created_home_spaces s
WHERE h.owner_id = s.creator_id
  AND h.space_id IS NULL;

INSERT INTO space_members (space_id, user_id, role)
SELECT h.space_id, h.owner_id, 'host'
FROM cloud_homes h
ON CONFLICT (space_id, user_id) DO UPDATE SET role = 'host';

ALTER TABLE cloud_homes
  ALTER COLUMN space_id SET NOT NULL;

CREATE INDEX cloud_homes_space_idx ON cloud_homes (space_id);
CREATE INDEX spaces_category_idx ON spaces (category) WHERE category IS NOT NULL;

COMMENT ON COLUMN cloud_homes.space_id IS
  'The canonical private Space owned by this user; cloud_homes adds home/life behavior.';
