-- Reset the drifted homepage hero copy to the "carrier" narrative.
--
-- The published/draft home config in the database had drifted to off-brand,
-- mismatched hero copy (a party-space line in Chinese, a different line in
-- English). The code default is correct, but the stored config overrides it.
-- This realigns hero eyebrow/title/body in BOTH draft and published configs to
-- the carrier ("we just carry a place's stories") narrative. Other blocks
-- (journey/guide/host) already match and are left untouched.
UPDATE site_page_configs
SET draft_config = jsonb_set(
        jsonb_set(
          jsonb_set(draft_config,
            '{hero,eyebrow}', '{"zh":"关于每一个真实地点","en":"About every real place"}'::jsonb, true),
          '{hero,title}', '{"zh":"我们不生产故事，我们只是地点故事的搬运工。","en":"We do not make the stories. We just carry a place''s stories to you."}'::jsonb, true),
        '{hero,body}', '{"zh":"每一个真实的地方，都有人来过、走过、留下过。我们把这些收下来，等你到了，推门进去，交给你。","en":"Every real place has people who came, walked it, and left something behind. We keep it here, so that when you arrive and step in, it is waiting for you."}'::jsonb, true),
    published_config = jsonb_set(
        jsonb_set(
          jsonb_set(published_config,
            '{hero,eyebrow}', '{"zh":"关于每一个真实地点","en":"About every real place"}'::jsonb, true),
          '{hero,title}', '{"zh":"我们不生产故事，我们只是地点故事的搬运工。","en":"We do not make the stories. We just carry a place''s stories to you."}'::jsonb, true),
        '{hero,body}', '{"zh":"每一个真实的地方，都有人来过、走过、留下过。我们把这些收下来，等你到了，推门进去，交给你。","en":"Every real place has people who came, walked it, and left something behind. We keep it here, so that when you arrive and step in, it is waiting for you."}'::jsonb, true),
    updated_at = now()
WHERE page_key = 'home';
