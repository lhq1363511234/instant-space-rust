-- Taste reframe: make the homepage speak in one clear cover message.
-- The editor can still change these fields later; this only resets the
-- published default away from the dense receipt-style hero.
UPDATE site_page_configs
SET draft_config = jsonb_set(
        jsonb_set(
          jsonb_set(
            jsonb_set(
              jsonb_set(draft_config,
                '{hero,eyebrow}', '{"zh":"inspace 是什么","en":"What inspace is"}'::jsonb, true),
              '{hero,title}', '{"zh":"走出屏幕，进入真实地点。","en":"Leave the screen. Enter the place."}'::jsonb, true),
            '{hero,body}', '{"zh":"到达之后，打开空间：看攻略、问现场、留下故事。","en":"After arrival, open the Space: read field notes, ask people there, and leave a story."}'::jsonb, true),
          '{hero,primary_label}', '{"zh":"探索空间","en":"Explore Spaces"}'::jsonb, true),
        '{hero,secondary_label}', '{"zh":"浏览攻略","en":"Browse guides"}'::jsonb, true),
    published_config = jsonb_set(
        jsonb_set(
          jsonb_set(
            jsonb_set(
              jsonb_set(published_config,
                '{hero,eyebrow}', '{"zh":"inspace 是什么","en":"What inspace is"}'::jsonb, true),
              '{hero,title}', '{"zh":"走出屏幕，进入真实地点。","en":"Leave the screen. Enter the place."}'::jsonb, true),
            '{hero,body}', '{"zh":"到达之后，打开空间：看攻略、问现场、留下故事。","en":"After arrival, open the Space: read field notes, ask people there, and leave a story."}'::jsonb, true),
          '{hero,primary_label}', '{"zh":"探索空间","en":"Explore Spaces"}'::jsonb, true),
        '{hero,secondary_label}', '{"zh":"浏览攻略","en":"Browse guides"}'::jsonb, true),
    updated_at = now()
WHERE page_key = 'home';
