INSERT INTO users (id, email, name, password_hash)
VALUES (
  '00000000-0000-0000-0000-000000000001',
  'host@example.com',
  'Demo Host',
  '$argon2id$v=19$m=19456,t=2,p=1$c2Vhc29uZWQtc2FsdA$BqYVx9v3m8JdtM3aK9LhCw'
);

INSERT INTO spaces (
  id, name_en, name_zh, space_type, province, city, district, lat, lng,
  tag_en, tag_zh, description_en, description_zh, is_public, password_hash,
  password_version, host_user_id, status
)
VALUES
(
  '10000000-0000-0000-0000-000000000001',
  'The Bund',
  '外滩',
  'scenic',
  '上海市',
  '上海市',
  '黄浦区',
  31.2397,
  121.4998,
  'Riverfront',
  '滨江',
  'Historic riverfront public space.',
  '历史滨江公共空间。',
  true,
  '$argon2id$v=19$m=19456,t=2,p=1$c2Vhc29uZWQtc2FsdA$BqYVx9v3m8JdtM3aK9LhCw',
  1,
  '00000000-0000-0000-0000-000000000001',
  'active'
),
(
  '10000000-0000-0000-0000-000000000002',
  'Private Tea Room',
  '私密茶室',
  'food',
  '浙江省',
  '杭州市',
  '西湖区',
  30.2496,
  120.1303,
  'Private',
  '私密',
  'Password protected community space.',
  '需要密码进入的社区空间。',
  false,
  '$argon2id$v=19$m=19456,t=2,p=1$c2Vhc29uZWQtc2FsdA$BqYVx9v3m8JdtM3aK9LhCw',
  1,
  '00000000-0000-0000-0000-000000000001',
  'active'
);

INSERT INTO guides (
  id, title_zh, title_en, summary_zh, summary_en, province, city, district,
  spot_name, status, featured, space_id, sections
)
VALUES (
  '20000000-0000-0000-0000-000000000001',
  '外滩导览',
  'The Bund Guide',
  '外滩地图首页验证导览。',
  'Guide used by the Rust homepage smoke path.',
  '上海市',
  '上海市',
  '黄浦区',
  '外滩',
  'published',
  true,
  '10000000-0000-0000-0000-000000000001',
  '[{"heading":"到达","body":"从南京东路步行到达。"}]'::jsonb
);

INSERT INTO locations (province, city, district, spot_name, source)
SELECT DISTINCT province, city, district, name_zh, 'spaces'
FROM spaces
WHERE province IS NOT NULL;

INSERT INTO locations (province, city, district, spot_name, source)
SELECT DISTINCT province, city, district, spot_name, 'guides'
FROM guides;
