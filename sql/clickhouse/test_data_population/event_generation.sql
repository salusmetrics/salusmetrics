-------------------------------------------------------------------------------
-- API KEYS and Sites
-------------------------------------------------------------------------------
-- TRUNCATE TABLE SALUS_METRICS.API_KEY;
-- INSERT INTO SALUS_METRICS.API_KEY
-- VALUES
--   ('167fac15-45aa-411e-85ab-c2d66cebdf20',
--   'salusmetrics.com',
--   'Salus Metrics LLC'),
--   ('d3225222-15aa-423d-afd4-fd59b343217c',
--   'test.salusmetrics.com',
--   'Salus Metrics LLC - Test Organization'),
--   ('bc9246bf-9dea-4564-87ec-9c2573d7966c',
--   'dev.salusmetrics.com',
--   'Salus Metrics LLC - Dev Team'),
--   ('510560e4-8660-430d-9e18-ce2c11134a21',
--   'demo.salusmetrics.com',
--   'Salus Metrics LLC - Sales Demo'),
--   ('abc-xyz',
--   '127.0.0.1',
--   'Salus Metrics LLC - Local Development'),
--   ('abc-xyz',
--   'localhost',
--   'Salus Metrics LLC - Local Development')

TRUNCATE TABLE SALUS_METRICS.VISITOR_EVENT;
-- TRUNCATE TABLE SALUS_METRICS.VISITOR_TIMESERIES;
TRUNCATE TABLE SALUS_METRICS.SESSION_EVENT;
-- TRUNCATE TABLE SALUS_METRICS.SESSION_TIMESERIES;
TRUNCATE TABLE SALUS_METRICS.SECTION_EVENT;
-- TRUNCATE TABLE SALUS_METRICS.SECTION_TIMESERIES;
TRUNCATE TABLE SALUS_METRICS.SECTION_COMBINED;

-- insert into EVENT select * from (select api_key, site, 'Session' as event_type, generateUUIDv7() as id, now() as ts, attrs from SESSION_EVENT limit 1);
-- insert into EVENT select * from (select api_key, site, 'Section' as event_type, generateUUIDv7() as id, now() as ts, attrs from SECTION_EVENT limit 1);


set param_BASE = 50000;
select {BASE:UInt32} as base_val;

-- Set up intermediary tables
-- Want to use asynchronous insert to get best use of resources.
-- Also use query cache for best performance SETTINGS use_query_cache = true

DROP TABLE IF EXISTS SALUS_METRICS.path_synth;

CREATE TABLE SALUS_METRICS.path_synth (
    `rv` UInt16,
    `path1` String,
    `path2` String,
    `resource` String,
    `title` String
) ENGINE = MergeTree
ORDER BY (rv);

INSERT INTO SALUS_METRICS.path_synth
SELECT (row_number() OVER ()) - 1 as rv, path1, path2, resource, concat(path1, path2, resource, ' - Title') AS Title
FROM (
  SELECT arrayJoin(['/', '/dir1/', '/dir2/', '/dir3/', '/dir4/', '/dir5/'] AS src) AS path1
) AS t1
CROSS JOIN (
  SELECT arrayJoin(['', 'subdir1/', 'subdir2/', 'subdir3/', 'subdir4/', 'subdir5/'] AS src) AS path2
) AS t2
CROSS JOIN (
  SELECT arrayJoin(['', 'type1', 'type2', 'type3', 'type4', 'type5'] AS src) AS resource
) AS t3;


DROP TABLE IF EXISTS SALUS_METRICS.browser_synth;

CREATE TABLE SALUS_METRICS.browser_synth (
    `rv` UInt16,
    `user_agent` String,
    `device_brand` String DEFAULT 'unknown',
    `device_model` String DEFAULT 'unknown',
    `os` String DEFAULT 'unknown',
    `os_version` String DEFAULT 'unknown',
    `browser` String DEFAULT 'unknown',
    `browser_version` String DEFAULT 'unknown'
) ENGINE = MergeTree
ORDER BY (rv);

with ua AS (
    SELECT (row_number() OVER ()) - 1 as rv, user_agent
    FROM (
    SELECT arrayJoin(
    ['Mozilla/5.0 (U; Linux i576 x86_64) Gecko/20100101 Firefox/62.8',
    'Mozilla/5.0 (Linux; U; Android 5.0.1; HTC Butterfly S 919d Build/LRX22G) AppleWebKit/601.17 (KHTML, like Gecko)  Chrome/52.0.3941.163 Mobile Safari/536.4',
    'Mozilla/5.0 (compatible; MSIE 10.0; Windows NT 6.0; WOW64 Trident/6.0)',
    'Mozilla/5.0 (Windows; U; Windows NT 10.4;; en-US) AppleWebKit/536.21 (KHTML, like Gecko) Chrome/51.0.2563.178 Safari/600.7 Edge/9.90933',
    'Mozilla/5.0 (iPhone; CPU iPhone OS 10_6_1; like Mac OS X) AppleWebKit/601.11 (KHTML, like Gecko)  Chrome/49.0.2511.277 Mobile Safari/603.5',
    'Mozilla/5.0 (Windows; U; Windows NT 10.3; WOW64) Gecko/20130401 Firefox/56.5',
    'Mozilla/5.0 (Macintosh; U; Intel Mac OS X 10_0_6) AppleWebKit/536.24 (KHTML, like Gecko) Chrome/51.0.2996.247 Safari/600',
    'Mozilla/5.0 (Windows; Windows NT 6.3; WOW64) AppleWebKit/602.35 (KHTML, like Gecko) Chrome/55.0.2839.293 Safari/533.8 Edge/10.35813',
    'Mozilla/5.0 (Macintosh; U; Intel Mac OS X 10_10_2; en-US) AppleWebKit/601.4 (KHTML, like Gecko) Chrome/53.0.3184.300 Safari/537',
    'Mozilla/5.0 (iPhone; CPU iPhone OS 9_3_7; like Mac OS X) AppleWebKit/601.12 (KHTML, like Gecko)  Chrome/47.0.3324.368 Mobile Safari/601.8',
    'Mozilla/5.0 (Windows; U; Windows NT 10.2; x64; en-US) AppleWebKit/603.15 (KHTML, like Gecko) Chrome/52.0.2427.326 Safari/535',
    'Mozilla/5.0 (iPhone; CPU iPhone OS 10_3_2; like Mac OS X) AppleWebKit/535.40 (KHTML, like Gecko)  Chrome/49.0.3660.141 Mobile Safari/600.0',
    'Mozilla/5.0 (Macintosh; U; Intel Mac OS X 8_7_8; en-US) Gecko/20130401 Firefox/65.6',
    'Mozilla/5.0 (Windows NT 10.4; WOW64) Gecko/20100101 Firefox/51.8',
    'Mozilla/5.0 (iPhone; CPU iPhone OS 8_6_4; like Mac OS X) AppleWebKit/537.7 (KHTML, like Gecko)  Chrome/54.0.3752.387 Mobile Safari/537.7',
    'Mozilla/5.0 (Windows; Windows NT 10.4; x64; en-US) AppleWebKit/602.6 (KHTML, like Gecko) Chrome/53.0.1232.186 Safari/603.2 Edge/9.84074',
    'Mozilla/5.0 (Android; Android 7.1.1; LG-H920 Build/NRD90C) AppleWebKit/603.38 (KHTML, like Gecko)  Chrome/54.0.2837.394 Mobile Safari/603.8',
    'Mozilla/5.0 (Linux; U; Linux i551 x86_64) AppleWebKit/601.8 (KHTML, like Gecko) Chrome/51.0.3050.314 Safari/536',
    'Mozilla/5.0 (iPhone; CPU iPhone OS 11_7_8; like Mac OS X) AppleWebKit/535.36 (KHTML, like Gecko)  Chrome/54.0.3868.157 Mobile Safari/603.4',
    'Mozilla/5.0 (U; Linux i566 ; en-US) AppleWebKit/537.9 (KHTML, like Gecko) Chrome/50.0.2930.303 Safari/537',
    'Mozilla/5.0 (Windows NT 6.0; WOW64) AppleWebKit/603.29 (KHTML, like Gecko) Chrome/53.0.2156.115 Safari/533.5 Edge/11.13812',
    'Mozilla/5.0 (Linux; Android 6.0.1; HTC One_M9 Build/MRA58K) AppleWebKit/537.47 (KHTML, like Gecko)  Chrome/53.0.1623.154 Mobile Safari/603.1',
    'Mozilla/5.0 (Windows; U; Windows NT 10.4;) AppleWebKit/603.6 (KHTML, like Gecko) Chrome/47.0.2296.377 Safari/603.0 Edge/8.82434',
    'Mozilla/5.0 (Macintosh; U; Intel Mac OS X 9_7_1) AppleWebKit/533.5 (KHTML, like Gecko) Chrome/47.0.3234.356 Safari/535',
    'Mozilla/5.0 (iPhone; CPU iPhone OS 8_3_8; like Mac OS X) AppleWebKit/602.37 (KHTML, like Gecko)  Chrome/53.0.2107.285 Mobile Safari/602.0',
    'Mozilla/5.0 (iPad; CPU iPad OS 11_8_7 like Mac OS X) AppleWebKit/535.11 (KHTML, like Gecko)  Chrome/48.0.1240.155 Mobile Safari/536.0',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 9_7_2; en-US) Gecko/20100101 Firefox/71.9',
    'Mozilla/5.0 (compatible; MSIE 9.0; Windows; Windows NT 6.0; Trident/5.0)',
    'Mozilla/5.0 (compatible; MSIE 7.0; Windows; Windows NT 6.0; WOW64; en-US Trident/4.0)',
    'Mozilla/5.0 (iPhone; CPU iPhone OS 10_6_5; like Mac OS X) AppleWebKit/600.34 (KHTML, like Gecko)  Chrome/55.0.3834.372 Mobile Safari/537.9',
    'Mozilla/5.0 (Windows NT 10.1;) AppleWebKit/537.44 (KHTML, like Gecko) Chrome/54.0.2604.111 Safari/536.1 Edge/15.14532',
    'Mozilla/5.0 (compatible; MSIE 11.0; Windows; U; Windows NT 6.0;; en-US Trident/7.0)',
    'Mozilla/5.0 (Linux; U; Android 6.0.1; HTC One M9 Build/MRA58K) AppleWebKit/603.10 (KHTML, like Gecko)  Chrome/53.0.1571.353 Mobile Safari/601.0',
    'Mozilla/5.0 (Windows; Windows NT 10.1;) AppleWebKit/534.23 (KHTML, like Gecko) Chrome/49.0.1872.228 Safari/533.8 Edge/9.45920',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 9_8_0; en-US) Gecko/20130401 Firefox/72.9',
    'Mozilla/5.0 (compatible; MSIE 7.0; Windows; U; Windows NT 6.0; WOW64; en-US Trident/4.0)',
    'Mozilla/5.0 (Linux; Android 4.4; Nexus_S_4G Build/GRJ22) AppleWebKit/533.39 (KHTML, like Gecko)  Chrome/54.0.3630.238 Mobile Safari/536.2',
    'Mozilla/5.0 (Linux; Android 4.4.4; LG-H200 Build/KOT49I) AppleWebKit/603.9 (KHTML, like Gecko)  Chrome/54.0.1700.142 Mobile Safari/536.5',
    'Mozilla/5.0 (Macintosh; U; Intel Mac OS X 10_7_2) AppleWebKit/600.19 (KHTML, like Gecko) Chrome/47.0.2981.134 Safari/600',
    'Mozilla/5.0 (Linux; U; Linux i563 ; en-US) Gecko/20100101 Firefox/55.6',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_10_1; en-US) AppleWebKit/603.24 (KHTML, like Gecko) Chrome/55.0.1125.177 Safari/535',
    'Mozilla/5.0 (compatible; MSIE 7.0; Windows; Windows NT 6.3;; en-US Trident/4.0)',
    'Mozilla/5.0 (Macintosh; U; Intel Mac OS X 7_8_5; en-US) Gecko/20100101 Firefox/54.5',
    'Mozilla/5.0 (compatible; MSIE 10.0; Windows; U; Windows NT 6.3; WOW64 Trident/6.0)',
    'Mozilla/5.0 (Windows; Windows NT 10.0; Win64; x64; en-US) AppleWebKit/536.1 (KHTML, like Gecko) Chrome/50.0.3785.365 Safari/534',
    'Mozilla/5.0 (iPhone; CPU iPhone OS 9_3_6; like Mac OS X) AppleWebKit/603.29 (KHTML, like Gecko)  Chrome/50.0.2923.339 Mobile Safari/602.8',
    'Mozilla/5.0 (compatible; MSIE 10.0; Windows; Windows NT 6.3; x64 Trident/6.0)',
    'Mozilla/5.0 (compatible; MSIE 11.0; Windows NT 6.2; Trident/7.0)',
    'Mozilla/5.0 (iPod; CPU iPod OS 11_4_6; like Mac OS X) AppleWebKit/534.13 (KHTML, like Gecko)  Chrome/50.0.3089.343 Mobile Safari/536.6',
    'Mozilla/5.0 (iPhone; CPU iPhone OS 7_4_3; like Mac OS X) AppleWebKit/537.30 (KHTML, like Gecko)  Chrome/50.0.2009.324 Mobile Safari/536.9',
    'Mozilla/5.0 (Linux; Android 5.1; MOTOROLA MOTO XT1575 Build/LXB22) AppleWebKit/535.44 (KHTML, like Gecko)  Chrome/47.0.1257.356 Mobile Safari/534.0',
    'Mozilla/5.0 (Macintosh; U; Intel Mac OS X 10_7_7) AppleWebKit/534.22 (KHTML, like Gecko) Chrome/51.0.2874.202 Safari/534',
    'Mozilla/5.0 (Linux x86_64; en-US) Gecko/20100101 Firefox/55.5',
    'Mozilla/5.0 (Linux; Android 4.4.1; LG-E989 Build/KOT49I) AppleWebKit/533.12 (KHTML, like Gecko)  Chrome/51.0.2979.223 Mobile Safari/534.4',
    'Mozilla/5.0 (U; Linux x86_64; en-US) Gecko/20130401 Firefox/56.9',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_4_6) Gecko/20130401 Firefox/45.3',
    'Mozilla/5.0 (Windows; Windows NT 10.1; WOW64; en-US) AppleWebKit/600.11 (KHTML, like Gecko) Chrome/48.0.2294.377 Safari/601.7 Edge/13.94984',
    'Mozilla/5.0 (Linux; U; Android 7.1; LG-H900 Build/NRD90C) AppleWebKit/534.17 (KHTML, like Gecko)  Chrome/47.0.3292.346 Mobile Safari/535.8',
    'Mozilla/5.0 (Linux; Linux x86_64) AppleWebKit/601.39 (KHTML, like Gecko) Chrome/52.0.2524.134 Safari/536',
    'Mozilla/5.0 (Macintosh; U; Intel Mac OS X 8_8_7; en-US) Gecko/20100101 Firefox/48.1',
    'Mozilla/5.0 (Linux; U; Linux i661 x86_64) Gecko/20130401 Firefox/45.6',
    'Mozilla/5.0 (Windows NT 10.5; Win64; x64; en-US) AppleWebKit/534.27 (KHTML, like Gecko) Chrome/47.0.1639.355 Safari/535.7 Edge/13.94497',
    'Mozilla/5.0 (Linux; Android 5.0.2; SAMSUNG SM-A700 Build/LMY47X) AppleWebKit/602.5 (KHTML, like Gecko)  Chrome/51.0.1883.224 Mobile Safari/602.4',
    'Mozilla/5.0 (compatible; MSIE 10.0; Windows; Windows NT 6.3; Win64; x64; en-US Trident/6.0)',
    'Mozilla/5.0 (iPhone; CPU iPhone OS 11_2_6; like Mac OS X) AppleWebKit/600.8 (KHTML, like Gecko)  Chrome/53.0.2808.172 Mobile Safari/601.8',
    'Mozilla/5.0 (Windows NT 10.3; x64; en-US) AppleWebKit/602.11 (KHTML, like Gecko) Chrome/52.0.1380.185 Safari/536.2 Edge/10.25758',
    'Mozilla/5.0 (Windows NT 10.0; x64) AppleWebKit/534.29 (KHTML, like Gecko) Chrome/49.0.1611.283 Safari/533.9 Edge/17.52518',
    'Mozilla/5.0 (Linux; Linux i651 x86_64; en-US) AppleWebKit/600.39 (KHTML, like Gecko) Chrome/47.0.3465.243 Safari/534',
    'Mozilla/5.0 (Linux; U; Android 5.0.1; SM-G901P Build/LRX22G) AppleWebKit/603.19 (KHTML, like Gecko)  Chrome/55.0.1647.367 Mobile Safari/534.2',
    'Mozilla/5.0 (Linux; Linux x86_64) Gecko/20100101 Firefox/49.1',
    'Mozilla/5.0 (Linux; U; Android 5.0; Lenovo A7000-a Build/LRX21M;) AppleWebKit/535.19 (KHTML, like Gecko)  Chrome/51.0.1421.352 Mobile Safari/600.0',
    'Mozilla/5.0 (U; Linux i650 x86_64) Gecko/20100101 Firefox/74.4',
    'Mozilla/5.0 (Android; Android 4.4; LG-V410 Build/KOT49I) AppleWebKit/533.41 (KHTML, like Gecko)  Chrome/48.0.2636.298 Mobile Safari/535.6',
    'Mozilla/5.0 (compatible; MSIE 7.0; Windows; Windows NT 6.0; x64 Trident/4.0)',
    'Mozilla/5.0 (iPad; CPU iPad OS 9_2_1 like Mac OS X) AppleWebKit/537.39 (KHTML, like Gecko)  Chrome/48.0.3345.296 Mobile Safari/534.6',
    'Mozilla/5.0 (Linux; U; Linux i661 x86_64) Gecko/20100101 Firefox/61.8',
    'Mozilla/5.0 (Windows; Windows NT 6.1; WOW64) AppleWebKit/601.18 (KHTML, like Gecko) Chrome/54.0.1767.399 Safari/603.1 Edge/17.47996',
    'Mozilla/5.0 (Windows; Windows NT 6.0; x64; en-US) AppleWebKit/601.47 (KHTML, like Gecko) Chrome/48.0.1653.369 Safari/534.7 Edge/12.65629',
    'Mozilla/5.0 (Macintosh; U; Intel Mac OS X 10_7_7) Gecko/20100101 Firefox/74.1',
    'Mozilla/5.0 (Windows NT 10.0;) AppleWebKit/600.31 (KHTML, like Gecko) Chrome/55.0.1823.115 Safari/601.5 Edge/16.95636',
    'Mozilla/5.0 (compatible; MSIE 9.0; Windows; U; Windows NT 10.2; WOW64; en-US Trident/5.0)',
    'Mozilla/5.0 (iPhone; CPU iPhone OS 10_2_0; like Mac OS X) AppleWebKit/600.9 (KHTML, like Gecko)  Chrome/50.0.2239.254 Mobile Safari/536.8',
    'Mozilla/5.0 (compatible; MSIE 10.0; Windows; Windows NT 6.2; x64 Trident/6.0)',
    'Mozilla/5.0 (Macintosh; U; Intel Mac OS X 7_9_5; en-US) AppleWebKit/601.33 (KHTML, like Gecko) Chrome/53.0.2649.156 Safari/601',
    'Mozilla/5.0 (Linux; U; Linux x86_64; en-US) AppleWebKit/602.1 (KHTML, like Gecko) Chrome/52.0.3654.250 Safari/534',
    'Mozilla/5.0 (iPhone; CPU iPhone OS 8_2_8; like Mac OS X) AppleWebKit/534.16 (KHTML, like Gecko)  Chrome/53.0.2596.286 Mobile Safari/536.9',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_3_8) Gecko/20100101 Firefox/70.0',
    'Mozilla/5.0 (Windows; U; Windows NT 6.2; WOW64) Gecko/20100101 Firefox/46.9',
    'Mozilla/5.0 (compatible; MSIE 7.0; Windows; U; Windows NT 10.0; x64; en-US Trident/4.0)',
    'Mozilla/5.0 (Linux; Android 5.0.2; Nokia 1000 4G Build/GRK39F) AppleWebKit/534.39 (KHTML, like Gecko)  Chrome/47.0.3879.302 Mobile Safari/603.1',
    'Mozilla/5.0 (compatible; MSIE 11.0; Windows; Windows NT 6.2; WOW64 Trident/7.0)',
    'Mozilla/5.0 (compatible; MSIE 8.0; Windows NT 6.0; x64 Trident/4.0)',
    'Mozilla/5.0 (U; Linux x86_64) Gecko/20130401 Firefox/50.7',
    'Mozilla/5.0 (Windows NT 6.0;; en-US) AppleWebKit/535.6 (KHTML, like Gecko) Chrome/47.0.3984.338 Safari/535',
    'Mozilla/5.0 (Windows; U; Windows NT 10.1;) AppleWebKit/536.49 (KHTML, like Gecko) Chrome/51.0.1811.155 Safari/535',
    'Mozilla/5.0 (compatible; MSIE 11.0; Windows; Windows NT 6.2; WOW64; en-US Trident/7.0)',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 7_7_5; en-US) AppleWebKit/537.41 (KHTML, like Gecko) Chrome/47.0.1693.152 Safari/602',
    'Mozilla/5.0 (Android; Android 6.0; HTC One_M9 Build/MRA58K) AppleWebKit/600.2 (KHTML, like Gecko)  Chrome/47.0.1343.128 Mobile Safari/601.6',
    'Mozilla/5.0 (Linux i682 x86_64) AppleWebKit/602.29 (KHTML, like Gecko) Chrome/50.0.3198.150 Safari/537',
    'Mozilla/5.0 (Macintosh; U; Intel Mac OS X 9_8_2) AppleWebKit/603.42 (KHTML, like Gecko) Chrome/55.0.1834.365 Safari/603']
    ) AS user_agent
    ) AS t1
),
ua_expanded AS (
    SELECT
        rv,
        user_agent,
        dictGet (
            'SALUS_METRICS.regexp_device',
            ('brand_replacement', 'device_replacement'),
            user_agent
        ) device_tuple,
        dictGet (
            'SALUS_METRICS.regexp_os',
            (
                'os_replacement',
                'os_v1_replacement',
                'os_v2_replacement',
                'os_v3_replacement'
            ),
            user_agent
        ) os_tuple,
        dictGet (
            'SALUS_METRICS.regexp_browser',
            (
                'family_replacement',
                'v1_replacement',
                'v2_replacement'
            ),
            user_agent
        ) as browser_tuple
    FROM ua
)
INSERT INTO SALUS_METRICS.browser_synth
SELECT
    rv,
    user_agent,
    tupleElement (device_tuple, 1) as device_brand,
    tupleElement (device_tuple, 2) as device_model,
    tupleElement (os_tuple, 1) as os,
    concat (
        tupleElement (os_tuple, 2),
        '.',
        tupleElement (os_tuple, 3),
        '.',
        tupleElement (os_tuple, 4)
    ) as os_version,
    tupleElement (browser_tuple, 1) as browser,
    concat (
        tupleElement (browser_tuple, 2),
        '.',
        tupleElement (browser_tuple, 3)
    ) as browser_version
FROM ua_expanded;


DROP TABLE IF EXISTS SALUS_METRICS.EVENT_synth;

CREATE TABLE SALUS_METRICS.EVENT_synth (
    `api_key` LowCardinality (String),
    `site` LowCardinality (String),
    `event_type` Enum8 (
        'Visitor' = 1,
        'Session' = 2,
        'Section' = 3,
        'Click' = 4
    ),
    `id` UUID,
    `ts` DateTime DEFAULT UUIDv7ToDateTime (id),
    `attrs` Map (LowCardinality (String), String),
) ENGINE = Null;

DROP TABLE IF EXISTS SALUS_METRICS.VISITOR_EVENT_synth;
CREATE TABLE SALUS_METRICS.VISITOR_EVENT_synth (
    `api_key` LowCardinality (String) CODEC (ZSTD (1)),
    `site` LowCardinality (String) CODEC (ZSTD (1)),
    `id` UUID CODEC (ZSTD (1)),
    `ts` DateTime CODEC (Delta (4), ZSTD (1)),
    `attrs` Map (LowCardinality (String), String) CODEC (ZSTD (1))
) ENGINE = MergeTree
ORDER BY
    (api_key, site, id);

DROP TABLE IF EXISTS SALUS_METRICS.visitor_event_mv_synth;
CREATE MATERIALIZED VIEW SALUS_METRICS.visitor_event_mv_synth TO SALUS_METRICS.VISITOR_EVENT_synth AS
SELECT
    api_key,
    site,
    id,
    ts,
    attrs
FROM
    SALUS_METRICS.EVENT_synth
WHERE
    event_type = 'Visitor'
    AND dictHas (
        'SALUS_METRICS.api_key_dictionary',
        (api_key, site)
    ) = 1;

DROP TABLE IF EXISTS SALUS_METRICS.SESSION_EVENT_synth;
CREATE TABLE SALUS_METRICS.SESSION_EVENT_synth (
    `api_key` LowCardinality (String) CODEC (ZSTD (1)),
    `site` LowCardinality (String) CODEC (ZSTD (1)),
    `id` UUID CODEC (ZSTD (1)),
    `ts` DateTime CODEC(Delta(4), ZSTD(1)),
    `parent` UUID ALIAS attrs['parent'],
    `device_brand` String DEFAULT 'unknown',
    `device_model` String DEFAULT 'unknown',
    `os` String DEFAULT 'unknown',
    `os_version` String DEFAULT 'unknown',
    `browser` String DEFAULT 'unknown',
    `browser_version` String DEFAULT 'unknown',
    `user_agent` String ALIAS attrs['user_agent'],
    `ipv4` Nullable(IPv4) ALIAS attrs['ipv4'],
    `ipv6` Nullable(IPv6) ALIAS attrs['ipv6'],
    `country_code` String,
    `state` String,
    `city` String,
    `attrs` Map (LowCardinality (String), String) CODEC (ZSTD (1))
) ENGINE = MergeTree
ORDER BY
    (api_key, site, id);

DROP TABLE IF EXISTS SALUS_METRICS.session_event_mv_synth;
CREATE MATERIALIZED VIEW SALUS_METRICS.session_event_mv_synth TO SALUS_METRICS.SESSION_EVENT_synth AS
SELECT
    api_key,
    site,
    id,
    ts,
    attrs,
    device_brand,
    device_model,
    os,
    os_version,
    browser,
    browser_version,
    COALESCE(tupleElement (loc_tuple, 1), 'unknown') as country_code,
    COALESCE(tupleElement (loc_tuple, 2), 'unknown') as state,
    COALESCE(tupleElement (loc_tuple, 3), 'unknown') as city
FROM (
    SELECT
        (row_number() OVER ()) - 1 as rv,
        api_key,
        site,
        id,
        ts,
        attrs,
        attrs['ipv4'] as ipv4,
        dictGetOrNull('SALUS_METRICS.dbip_city_ipv4_trie',
            ('country_code', 'state', 'city', 'latitude', 'longitude'), coalesce(toIPv4(ipv4), toIPv4(0))) as loc_tuple
    FROM SALUS_METRICS.EVENT_synth
    WHERE
        event_type = 'Session'
        AND dictHas (
            'SALUS_METRICS.api_key_dictionary',
            (api_key, site)
        ) = 1
        AND attrs['parent'] > ''
) AS ev
INNER JOIN SALUS_METRICS.browser_synth ON (ev.rv % 100 = SALUS_METRICS.browser_synth.rv);


DROP TABLE IF EXISTS SALUS_METRICS.SECTION_EVENT_synth;
CREATE TABLE SALUS_METRICS.SECTION_EVENT_synth (
    `api_key` LowCardinality (String) CODEC (ZSTD (1)),
    `site` LowCardinality (String) CODEC (ZSTD (1)),
    `path` String CODEC (ZSTD (1)),
    `query` String ALIAS queryString(attrs['location']),
    `fragment` String ALIAS fragment(attrs['location']),
    `title` String ALIAS attrs['title'],
    `id` UUID CODEC (ZSTD (1)),
    `ts` DateTime CODEC(Delta(4), ZSTD(1)),
    `parent` UUID CODEC(ZSTD(1)),
    `attrs` Map (LowCardinality (String), String) CODEC (ZSTD (1))
) ENGINE = MergeTree
ORDER BY
    (api_key, site, parent, id)
;

DROP TABLE IF EXISTS SALUS_METRICS.section_event_mv_synth;
CREATE MATERIALIZED VIEW SALUS_METRICS.section_event_mv_synth TO SALUS_METRICS.SECTION_EVENT_synth AS
SELECT
    api_key,
    site,
    path(attrs['location']) as path,
    id,
    ts,
    toUUID(attrs['parent']) as parent,
    attrs
FROM
    SALUS_METRICS.EVENT_synth
WHERE
    event_type = 'Section'
    AND dictHas (
        'SALUS_METRICS.api_key_dictionary',
        (api_key, site)
    ) = 1
    AND attrs['parent'] > '';


-------------------------------------------------------------------------------
-- Visitor Event Random Data Population
-------------------------------------------------------------------------------
-- **** Start with uniform distribution over the past year ****
WITH apikeys AS (
  SELECT api_key, site
  FROM SALUS_METRICS.API_KEY
  WHERE site = 'salusmetrics.com'
)
INSERT INTO SALUS_METRICS.EVENT_synth
SELECT api_key, site, 'Visitor',
    generateUUIDv7(n.number),
    (now() - toIntervalMinute(randUniform(0, 60 * 24))) - toIntervalDay(randUniform(0, 365)) AS ts,
    map('lang','en/US') AS attrs
FROM apikeys
CROSS JOIN numbers({BASE:UInt16} * 10) n;

-- **** Add records to represent days with high visitor signups due to some external event ****
WITH apikeys AS (
  SELECT api_key, site
  FROM SALUS_METRICS.API_KEY
  WHERE site = 'salusmetrics.com'
)
INSERT INTO SALUS_METRICS.EVENT_synth
SELECT *
FROM
(
    SELECT api_key, site, 'Visitor',
        generateUUIDv7(n.number),
        (now() - toIntervalMinute(randUniform(0, 60 * 24))) - toIntervalDay(320 - randExponential(1 / 1.3)) AS ts,
        map('lang','en/US') AS attrs
    FROM apikeys
    CROSS JOIN numbers({BASE:UInt16} * 4) n
    UNION ALL
    SELECT api_key, site, 'Visitor',
        generateUUIDv7(n.number),
        (now() - toIntervalMinute(randUniform(0, 60 * 24))) - toIntervalDay(177 - randExponential(1 / 1.3)) AS ts,
        map('lang','en/US') AS attrs
    FROM apikeys
    CROSS JOIN numbers({BASE:UInt16} * 3) n
    UNION ALL
    SELECT api_key, site, 'Visitor',
        generateUUIDv7(n.number),
        (now() - toIntervalMinute(randUniform(0, 60 * 24))) - toIntervalDay(48 - randExponential(1 / 1.3)) AS ts,
        map('lang','en/US') AS attrs
    FROM apikeys
    CROSS JOIN numbers({BASE:UInt16} * 3) n
);


-------------------------------------------------------------------------------
-- Session Event Random Data Population
-------------------------------------------------------------------------------
-- **** Insert records for each child with the same ts as the parent for all types ****
WITH visitors AS (
  SELECT api_key, site, id as visitor_id, ts as visitor_ts, row_number() OVER () as rv
  FROM SALUS_METRICS.VISITOR_EVENT_synth
),
attr AS (
  SELECT user_agent, ipv4, row_number() OVER() as ra
  FROM (
    SELECT user_agent, rand32() as ipv4
    FROM visitors
    INNER JOIN SALUS_METRICS.browser_synth
    ON (visitors.rv % 100 = SALUS_METRICS.browser_synth.rv)
  )
)
INSERT INTO SALUS_METRICS.EVENT_synth
SELECT api_key, site, 'Session', generateUUIDv7(rv) as id, visitor_ts as ts,
  map('parent', toString(visitor_id), 'user_agent', user_agent, 'ipv4', IPv4NumToString(ipv4)) as attrs
FROM attr
INNER JOIN visitors ON attr.ra = visitors.rv
ORDER BY (api_key, site, id)
SETTINGS join_algorithm = 'full_sorting_merge';

-- **** Insert randomly distributed records that are bounded by the parent ts ****
WITH visitors AS (
  SELECT api_key, site, id as visitor_id, ts as visitor_ts, (now() - ts) as diff, row_number() OVER () as rv
  FROM SALUS_METRICS.VISITOR_EVENT_synth
),
attr AS (
  SELECT user_agent, ipv4,
    toUInt64(
        floor(
            randNormal(
                ({BASE:UInt16} * 20) / 2,
                ({BASE:UInt16} * 20) / 3))
        ) % ({BASE:UInt16} * 20) + 1 as ra
  FROM (
    SELECT user_agent, rand32() as ipv4
    FROM visitors
    INNER JOIN SALUS_METRICS.browser_synth
    ON (visitors.rv % 100 = SALUS_METRICS.browser_synth.rv)
    CROSS JOIN numbers(9) as mult
  )
)
INSERT INTO SALUS_METRICS.EVENT_synth
SELECT api_key, site, 'Session', generateUUIDv7(rv) as id, (now() - toIntervalSecond(randUniform(0, 31536000) % diff)) as ts,
  map('parent', toString(visitor_id), 'user_agent', user_agent, 'ipv4', IPv4NumToString(ipv4)) as attrs
FROM attr
INNER JOIN visitors ON attr.ra = visitors.rv
ORDER BY (api_key, site, id)
SETTINGS join_algorithm = 'full_sorting_merge';



-------------------------------------------------------------------------------
-- Section Event Random Data Population
-------------------------------------------------------------------------------
-- **** Insert records for each child with the same ts as the parent for all types ****
WITH sessions AS (
  SELECT api_key, site, id as session_id, ts as session_ts, row_number() OVER() as rv
  FROM SALUS_METRICS.SESSION_EVENT_synth
),
attr AS (
    SELECT *, row_number() OVER() as ra
    FROM (
        SELECT floor(abs(randNormal(0, 55))) % (SELECT count(*) FROM SALUS_METRICS.path_synth) AS path_id
        FROM sessions
    ) as rands
    INNER JOIN SALUS_METRICS.path_synth
    ON SALUS_METRICS.path_synth.rv = rands.path_id
)
INSERT INTO SALUS_METRICS.EVENT_synth
SELECT api_key, site, 'Section', generateUUIDv7(rv) as id, session_ts as ts,
  map('parent', toString(session_id), 'location', 'https://salusmetrics.com' || path1 || path2 || resource, 'title', title) as attrs
FROM attr
INNER JOIN sessions ON attr.ra = sessions.rv
ORDER BY (api_key, site, id)
SETTINGS join_algorithm = 'full_sorting_merge';

-- **** Insert randomly distributed records that are bounded by the parent ts ****
WITH sessions AS (
  SELECT api_key, site, id as session_id, ts as session_ts, row_number() OVER (ORDER BY id) as rv
  FROM SALUS_METRICS.SESSION_EVENT_synth
),
attr AS (
  SELECT *,
  toUInt64(
      floor(
          randUniform(
              0,
              ({BASE:UInt16} * 200)))
      ) + 1 as ra
  FROM (
    SELECT *
    FROM (
        SELECT floor(abs(randNormal(0, 55))) % (SELECT count(*) FROM SALUS_METRICS.path_synth) AS path_id
        FROM sessions
        CROSS JOIN numbers(9) as mult
    ) as rands
    INNER JOIN SALUS_METRICS.path_synth
    ON SALUS_METRICS.path_synth.rv = rands.path_id
  )
)
INSERT INTO SALUS_METRICS.EVENT_synth
SELECT api_key, site, 'Section', generateUUIDv7(rv) as id, (session_ts + toIntervalSecond(randUniform(0, 3600))) as ts,
    map('parent', toString(session_id), 'location', 'https://salusmetrics.com' || path1 || path2 || resource, 'title', title) as attrs
FROM attr
INNER JOIN sessions ON attr.ra = sessions.rv
ORDER BY (api_key, site, id)
SETTINGS join_algorithm = 'full_sorting_merge';

INSERT INTO SALUS_METRICS.VISITOR_EVENT
SELECT * FROM SALUS_METRICS.VISITOR_EVENT_synth;
DROP TABLE IF EXISTS SALUS_METRICS.visitor_event_mv_synth;
DROP TABLE IF EXISTS SALUS_METRICS.VISITOR_EVENT_synth;

INSERT INTO SALUS_METRICS.SESSION_EVENT
SELECT * FROM SALUS_METRICS.SESSION_EVENT_synth;
DROP TABLE IF EXISTS SALUS_METRICS.session_event_mv_synth;
DROP TABLE IF EXISTS SALUS_METRICS.SESSION_EVENT_synth;

INSERT INTO SALUS_METRICS.SECTION_EVENT
SELECT * FROM SALUS_METRICS.SECTION_EVENT_synth;
DROP TABLE IF EXISTS SALUS_METRICS.section_event_mv_synth;
DROP TABLE IF EXISTS SALUS_METRICS.SECTION_EVENT_synth;

DROP TABLE IF EXISTS SALUS_METRICS.path_synth;
DROP TABLE IF EXISTS SALUS_METRICS.browser_synth;
