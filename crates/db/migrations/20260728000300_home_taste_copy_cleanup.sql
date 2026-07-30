-- Taste-skill copy cleanup: remove em-dashes from stored homepage strings so
-- the database cannot override the cleaned code defaults with punctuation that
-- reads as AI-styled marketing copy. This is safe for JSON because only string
-- content is changed; JSON structure and keys stay intact.
UPDATE site_page_configs
SET draft_config = replace(draft_config::text, '—', ',')::jsonb,
    published_config = replace(published_config::text, '—', ',')::jsonb,
    updated_at = now()
WHERE page_key = 'home';
