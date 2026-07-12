-- Hong Kong and Macao are Special Administrative Regions of China,
-- not countries. Remap their places under China as admin1 regions,
-- mirroring the Taiwan treatment in 20260711000800.

-- Demote any capital-class markers to regional (PPLA).
UPDATE geo_places
SET feature_code = 'PPLA'
WHERE (country_code IN ('HK', 'MO')
       OR country_name ILIKE 'Hong Kong'
       OR country_name ILIKE 'Macao'
       OR country_name ILIKE 'Macau')
  AND feature_code = 'PPLC';

-- Reassign Hong Kong places to China / Hong Kong province.
UPDATE geo_places
SET
  country_name = 'China',
  country_code = 'CN',
  admin1_name = 'Hong Kong'
WHERE country_code = 'HK' OR country_name ILIKE 'Hong Kong';

-- Reassign Macao places to China / Macao province.
UPDATE geo_places
SET
  country_name = 'China',
  country_code = 'CN',
  admin1_name = 'Macao'
WHERE country_code = 'MO' OR country_name ILIKE 'Macao' OR country_name ILIKE 'Macau';

-- Drop the standalone capital rows so fly-to never treats them as countries.
DELETE FROM geo_capitals
WHERE country_name ILIKE 'Hong Kong'
   OR country_name ILIKE 'Macao'
   OR country_name ILIKE 'Macau';
