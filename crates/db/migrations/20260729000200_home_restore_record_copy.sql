-- Restore the homepage version explicitly requested by the product owner:
-- "让世界也值得被记录" with the right-side place record sheet.
-- This reverses the rejected single-cover poster copy without changing routes
-- or homepage editor structure.

WITH patch AS (
  SELECT
    '{
      "title": {"zh":"inspace｜让世界也值得被记录","en":"inspace | Let the world be worth recording"},
      "description": {"zh":"每一个真实地点，都有人来过、走过、留下过。inspace 把这些现场经验、攻略和故事收在地点名下。","en":"Every real place has been visited, crossed, and remembered. inspace keeps field notes, guides, and stories under the place itself."}
    }'::jsonb AS seo_patch,
    '{
      "eyebrow": {"zh":"关于每一个真实地点","en":"About every real place"},
      "title": {"zh":"让世界也值得被记录","en":"Let the world be worth recording"},
      "body": {"zh":"每一个真实的地方，都有人来过、走过、留下过。我们把这些收下来，等你到了，推门进去，交给你。","en":"Every real place has people who came, walked it, and left something behind. We keep it here, so that when you arrive and step in, it is waiting for you."},
      "primary_label": {"zh":"查找一个空间","en":"Find a Space"},
      "secondary_label": {"zh":"浏览空间攻略","en":"Browse Space guides"},
      "sample_location": {"zh":"上海 · 黄浦区","en":"Shanghai · Huangpu"},
      "sample_title": {"zh":"今晚去外滩，怎么走、哪里人少？","en":"Heading to the Bund tonight, which route is easier and less crowded?"},
      "sample_body": {"zh":"先看路线、交通和避坑，再查看刚刚发生的现场变化。","en":"Check routes, transit, and pitfalls, then see what has just changed on site."},
      "sample_guide_label": {"zh":"空间攻略 · 外滩夜景与人流","en":"Space guide · Night views and crowds"},
      "sample_question": {"zh":"南京东路站从哪个出口走更近？","en":"Which East Nanjing Road exit is the shortest walk?"},
      "sample_presence": {"zh":"12 人在线 · 3 条现场更新","en":"12 online · 3 live updates"}
    }'::jsonb AS hero_patch
)
UPDATE site_page_configs AS s
SET
  draft_config = jsonb_set(
    jsonb_set(s.draft_config, '{seo}', COALESCE(s.draft_config->'seo', '{}'::jsonb) || patch.seo_patch, true),
    '{hero}', COALESCE(s.draft_config->'hero', '{}'::jsonb) || patch.hero_patch, true
  ),
  published_config = jsonb_set(
    jsonb_set(s.published_config, '{seo}', COALESCE(s.published_config->'seo', '{}'::jsonb) || patch.seo_patch, true),
    '{hero}', COALESCE(s.published_config->'hero', '{}'::jsonb) || patch.hero_patch, true
  ),
  published_version = s.published_version + 1,
  updated_at = now()
FROM patch
WHERE s.page_key = 'home'
  AND (s.published_config->'hero'->'title'->>'zh')
      IS DISTINCT FROM '让世界也值得被记录';
