-- Align the currently published homepage with the local Space host campaign.
UPDATE site_page_configs
SET draft_config = jsonb_set(
        jsonb_set(
          jsonb_set(draft_config, '{host,title}', '{"zh":"地图上已经有很多空间，正在等真正熟悉它的人。","en":"Many Spaces are already on the map, waiting for someone who truly knows them."}'::jsonb, true),
          '{host,body}', '{"zh":"系统可以先点亮坐标，只有当地人能让它长期可信。我们正在招募空间主理人：整理攻略、回答现场问题、保留这里的故事。","en":"A system can light a coordinate; only a local can keep it trustworthy. We are recruiting Space hosts to maintain guides, answer on-site questions, and keep local stories."}'::jsonb, true),
        '{host,cta_label}', '{"zh":"成为空间主理人","en":"Become a Space host"}'::jsonb, true),
    published_config = jsonb_set(
        jsonb_set(
          jsonb_set(published_config, '{host,title}', '{"zh":"地图上已经有很多空间，正在等真正熟悉它的人。","en":"Many Spaces are already on the map, waiting for someone who truly knows them."}'::jsonb, true),
          '{host,body}', '{"zh":"系统可以先点亮坐标，只有当地人能让它长期可信。我们正在招募空间主理人：整理攻略、回答现场问题、保留这里的故事。","en":"A system can light a coordinate; only a local can keep it trustworthy. We are recruiting Space hosts to maintain guides, answer on-site questions, and keep local stories."}'::jsonb, true),
        '{host,cta_label}', '{"zh":"成为空间主理人","en":"Become a Space host"}'::jsonb, true),
    updated_at = now()
WHERE page_key = 'home';
