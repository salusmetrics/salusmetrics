-------------------------------------------------------------------------------
-- API KEYS and Sites
-------------------------------------------------------------------------------
-- TRUNCATE TABLE SALUS_METRICS.API_KEY;
INSERT INTO SALUS_METRICS.API_KEY
VALUES
  ('167fac15-45aa-411e-85ab-c2d66cebdf20',
  'salusmetrics.com',
  'Salus Metrics LLC'),
  ('d3225222-15aa-423d-afd4-fd59b343217c',
  'test.salusmetrics.com',
  'Salus Metrics LLC - Test Organization'),
  ('bc9246bf-9dea-4564-87ec-9c2573d7966c',
  'dev.salusmetrics.com',
  'Salus Metrics LLC - Dev Team'),
  ('510560e4-8660-430d-9e18-ce2c11134a21',
  'demo.salusmetrics.com',
  'Salus Metrics LLC - Sales Demo')


-------------------------------------------------------------------------------
-- Visitor Event Random Data Population
-------------------------------------------------------------------------------
-- TRUNCATE TABLE SALUS_METRICS.EVENT;
-- TRUNCATE TABLE SALUS_METRICS.VISITOR_EVENT;
-- **** Start with uniform distribution over the past year ****
WITH apikeys AS (
  SELECT api_key, site
  FROM SALUS_METRICS.API_KEY
  WHERE site = 'salusmetrics.com'
)
INSERT INTO SALUS_METRICS.EVENT
SELECT api_key, site, 'Visitor',
    generateUUIDv7(n.number),
    (now() - toIntervalMinute(randUniform(0, 60 * 24))) - toIntervalDay(randUniform(0, 365)) AS ts,
    map('lang','en/US') AS attrs
FROM apikeys
CROSS JOIN numbers(50000) n;

-- **** Add records to represent days with high visitor signups due to some external event ****
WITH apikeys AS (
  SELECT api_key, site
  FROM SALUS_METRICS.API_KEY
  WHERE site = 'salusmetrics.com'
)
INSERT INTO SALUS_METRICS.EVENT
SELECT *
FROM
(
    SELECT api_key, site, 'Visitor',
        generateUUIDv7(n.number),
        (now() - toIntervalMinute(randUniform(0, 60 * 24))) - toIntervalDay(320 - randExponential(1 / 1.3)) AS ts,
        map('lang','en/US') AS attrs
    FROM apikeys
    CROSS JOIN numbers(15000) n
    UNION ALL
    SELECT api_key, site, 'Visitor',
        generateUUIDv7(n.number),
        (now() - toIntervalMinute(randUniform(0, 60 * 24))) - toIntervalDay(177 - randExponential(1 / 1.3)) AS ts,
        map('lang','en/US') AS attrs
    FROM apikeys
    CROSS JOIN numbers(7000) n
    UNION ALL
    SELECT api_key, site, 'Visitor',
        generateUUIDv7(n.number),
        (now() - toIntervalMinute(randUniform(0, 60 * 24))) - toIntervalDay(48 - randExponential(1 / 1.3)) AS ts,
        map('lang','en/US') AS attrs
    FROM apikeys
    CROSS JOIN numbers(28000) n
)



-------------------------------------------------------------------------------
-- Session Event Random Data Population
-------------------------------------------------------------------------------
-- **** Insert records for each child with the same ts as the parent for all types ****
WITH visitors AS (
  SELECT api_key, site, id as visitor_id, ts as visitor_ts, row_number() OVER (ORDER BY id) as rv
  FROM SALUS_METRICS.VISITOR_EVENT
),
attr AS (
  SELECT browser, country, language, row_number() OVER() as ra
  FROM (
    SELECT *
    FROM generateRandom(
      'browser Enum8(''Chrome'', ''Firefox'', ''Edge'', ''Safari''),
      country Enum8(''United States'', ''Canada'', ''Mexico'', ''United Kingdom''),
      language Enum8(''English'', ''French'', ''Spanish'', ''Arabic'')')
    LIMIT (100000)
  )
)
INSERT INTO SALUS_METRICS.EVENT
SELECT api_key, site, 'Session', generateUUIDv7(rv) as id, visitor_ts as ts,
  map('parent', toString(visitor_id), 'browser', browser, 'country', country, 'language', language) as attrs
FROM visitors
INNER JOIN attr ON attr.ra = visitors.rv;

-- **** Insert randomly distributed records that are bounded by the parent ts ****
WITH visitors AS (
  SELECT api_key, site, id as visitor_id, ts as visitor_ts, (now() - ts) as diff, row_number() OVER (ORDER BY id) as rv
  FROM SALUS_METRICS.VISITOR_EVENT
),
attr AS (
  SELECT browser, country, language, toUInt64(floor(randNormal(50000, 15000))) % 100000 + 1 as ra
  FROM (
    SELECT *
    FROM generateRandom(
      'browser Enum8(''Chrome'', ''Firefox'', ''Edge'', ''Safari''),
      country Enum8(''United States'', ''Canada'', ''Mexico'', ''United Kingdom''),
      language Enum8(''English'', ''French'', ''Spanish'', ''Arabic'')')
    LIMIT (900000)
  )
)
INSERT INTO SALUS_METRICS.EVENT
SELECT api_key, site, 'Session', generateUUIDv7(rv) as id, (now() - toIntervalSecond(randUniform(0, 31536000) % diff)) as ts,
  map('parent', toString(visitor_id), 'browser', browser, 'country', country, 'language', language) as attrs
FROM visitors
INNER JOIN attr ON attr.ra = visitors.rv



-------------------------------------------------------------------------------
-- Section Event Random Data Population
-------------------------------------------------------------------------------
-- **** Insert records for each child with the same ts as the parent for all types ****
WITH sessions AS (
  SELECT api_key, site, id as session_id, ts as session_ts, row_number() OVER (ORDER BY id) as rv
  FROM SALUS_METRICS.SESSION_EVENT
),
attr AS (
  SELECT *, row_number() OVER() as ra
  FROM (
    SELECT *
    FROM generateRandom(
      'path1 Enum8(''/'', ''/world-maps/'', ''/asia/'', ''/africa/'', ''/antarctica/'', ''/oceania/'', ''/northamerica/'', ''/southamerica/'', ''/mars/''),
      path2 Enum8('''', ''country1'', ''country2'', ''country3'', ''country4'', ''country5'', ''country6'', ''country7'', ''country8'', ''country9''),
      resource Enum8(''type1'', ''type2'', ''type3'', ''type4'', ''type5'', ''type6'', ''type7''),
      query String,
      title String')
    LIMIT (1000000)
  )
)
INSERT INTO SALUS_METRICS.EVENT
SELECT api_key, site, 'Section', generateUUIDv7(rv) as id, session_ts as ts,
  map('parent', toString(session_id), 'path', path1 || path2, 'resource', resource, 'query', query, 'title', title) as attrs
FROM sessions
INNER JOIN attr ON attr.ra = sessions.rv;

-- **** Insert randomly distributed records that are bounded by the parent ts ****
WITH sessions AS (
  SELECT api_key, site, id as session_id, ts as session_ts, (now() - ts) as diff, row_number() OVER (ORDER BY id) as rv
  FROM SALUS_METRICS.SESSION_EVENT
),
attr AS (
  SELECT *, toUInt64(floor(randNormal(500000, 150000))) % 1000000 + 1 as ra
  FROM (
    SELECT *
    FROM generateRandom(
      'path1 Enum8(''/'', ''/world-maps/'', ''/asia/'', ''/africa/'', ''/antarctica/'', ''/oceania/'', ''/northamerica/'', ''/southamerica/'', ''/mars/''),
      path2 Enum8('''', ''country1'', ''country2'', ''country3'', ''country4'', ''country5'', ''country6'', ''country7'', ''country8'', ''country9''),
      resource Enum8(''type1'', ''type2'', ''type3'', ''type4'', ''type5'', ''type6'', ''type7''),
      query String,
      title String')
    LIMIT (9000000)
  )
)
INSERT INTO SALUS_METRICS.EVENT
SELECT api_key, site, 'Section', generateUUIDv7(rv) as id, (session_ts + toIntervalSecond(randUniform(0, 3600))) as ts,
  map('parent', toString(session_id), 'path', path1 || path2, 'resource', resource, 'query', query, 'title', title) as attrs
FROM sessions
INNER JOIN attr ON attr.ra = sessions.rv
