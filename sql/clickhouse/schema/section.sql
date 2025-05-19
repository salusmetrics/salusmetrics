DROP TABLE IF EXISTS SALUS_METRICS.SECTION_EVENT;

CREATE TABLE SALUS_METRICS.SECTION_EVENT (
    `api_key` LowCardinality (String) CODEC (ZSTD),
    `site` LowCardinality (String) CODEC (ZSTD),
    `path` String CODEC (ZSTD),
    `query` String ALIAS queryString(attrs['location']),
    `fragment` String ALIAS fragment(attrs['location']),
    `title` String ALIAS attrs['title'],
    `id` UUID CODEC (ZSTD),
    `ts` DateTime CODEC(Delta, ZSTD),
    `parent` UUID CODEC(ZSTD),
    `attrs` Map (LowCardinality (String), String) CODEC (ZSTD)
) ENGINE = MergeTree
ORDER BY
    (api_key, site, id)
TTL ts + INTERVAL 1 WEEK
;


DROP TABLE IF EXISTS SALUS_METRICS.section_event_mv;

CREATE MATERIALIZED VIEW SALUS_METRICS.section_event_mv TO SALUS_METRICS.SECTION_EVENT AS
SELECT
    api_key,
    site,
    path(attrs['location']) as path,
    id,
    ts,
    toUUID(attrs['parent']) as parent,
    attrs
FROM
    SALUS_METRICS.EVENT
WHERE
    event_type = 'Section'
    AND dictHas (
        'SALUS_METRICS.api_key_dictionary',
        (api_key, site)
    ) = 1
    AND attrs['parent'] > ''
    ORDER BY
        (api_key, site, parent, id)
;



-- ---------------------------------------------------------------------------
-- ---------------------------------------------------------------------------

DROP TABLE IF EXISTS SALUS_METRICS.SECTION_COMBINED;

CREATE TABLE SALUS_METRICS.SECTION_COMBINED (
    `api_key` LowCardinality (String) CODEC (ZSTD),
    `site` LowCardinality (String) CODEC (ZSTD),
    `ts` DateTime CODEC (Delta, ZSTD),
    `path` String CODEC (ZSTD),
    `os` LowCardinality (String) CODEC (ZSTD),
    `browser` LowCardinality (String) CODEC (ZSTD),
    `country_code` LowCardinality (String) CODEC (ZSTD),
    `city` String CODEC (ZSTD),
    -- `visitor` UUID CODEC (ZSTD)
    `visitor` UInt32 CODEC (ZSTD)
    -- ,
    -- PROJECTION section_browser_projection (
    --     SELECT *
    --     ORDER BY (api_key, site, browser, ts)
    -- )
    -- ,
    -- PROJECTION section_city_projection (
    --     SELECT *
    --     ORDER BY (api_key, site, city, ts)
    -- )
    -- ,
    -- PROJECTION section_country_projection (
    --     SELECT *
    --     ORDER BY (api_key, site, country_code, ts)
    -- )
    -- ,
    -- PROJECTION section_os_projection (
    --     SELECT *
    --     ORDER BY (api_key, site, os, ts)
    -- )
    -- ,
    -- PROJECTION section_path_projection (
    --     SELECT *
    --     ORDER BY (api_key, site, path, ts)
    -- )
) ENGINE = MergeTree
ORDER BY (
    api_key,
    site,
    -- path,
    ts);

-- ALTER TABLE SALUS_METRICS.SECTION_COMBINED DROP PROJECTION IF EXISTS section_daily_browser_projection;
-- ALTER TABLE SALUS_METRICS.SECTION_COMBINED ADD PROJECTION section_daily_browser_projection (
--     SELECT
--         api_key,
--         site,
--         browser,
--         toStartOfDay(ts),
--         COUNT(*)
--     -- FROM SALUS_METRICS.SECTION_COMBINED
--     GROUP BY api_key, site, browser, toStartOfDay(ts)
-- );
-- ALTER TABLE SALUS_METRICS.SECTION_COMBINED MATERIALIZE PROJECTION section_daily_browser_projection;

-- ALTER TABLE SALUS_METRICS.SECTION_COMBINED DROP PROJECTION IF EXISTS section_daily_city_projection;
-- ALTER TABLE SALUS_METRICS.SECTION_COMBINED ADD PROJECTION section_daily_city_projection (
--     SELECT
--         api_key,
--         site,
--         city,
--         toStartOfDay(ts),
--         COUNT(*)
--     -- FROM SALUS_METRICS.SECTION_COMBINED
--     GROUP BY api_key, site, city, toStartOfDay(ts)
-- );
-- ALTER TABLE SALUS_METRICS.SECTION_COMBINED MATERIALIZE PROJECTION section_daily_city_projection;

-- ALTER TABLE SALUS_METRICS.SECTION_COMBINED DROP PROJECTION IF EXISTS section_daily_country_projection;
-- ALTER TABLE SALUS_METRICS.SECTION_COMBINED ADD PROJECTION section_daily_country_projection (
--     SELECT
--         api_key,
--         site,
--         country_code,
--         toStartOfDay(ts),
--         COUNT(*),
--         uniqCombined(visitor)
--     -- FROM SALUS_METRICS.SECTION_COMBINED
--     GROUP BY api_key, site, country_code, toStartOfDay(ts)
-- );
-- ALTER TABLE SALUS_METRICS.SECTION_COMBINED MATERIALIZE PROJECTION section_daily_country_projection;

-- ALTER TABLE SALUS_METRICS.SECTION_COMBINED DROP PROJECTION IF EXISTS section_daily_os_projection;
-- ALTER TABLE SALUS_METRICS.SECTION_COMBINED ADD PROJECTION section_daily_os_projection (
--     SELECT
--         api_key,
--         site,
--         os,
--         toStartOfDay(ts),
--         COUNT(*)
--     -- FROM SALUS_METRICS.SECTION_COMBINED
--     GROUP BY api_key, site, os, toStartOfDay(ts)
-- );
-- ALTER TABLE SALUS_METRICS.SECTION_COMBINED MATERIALIZE PROJECTION section_daily_os_projection;

-- ALTER TABLE SALUS_METRICS.SECTION_COMBINED DROP PROJECTION IF EXISTS section_daily_path_projection;
-- ALTER TABLE SALUS_METRICS.SECTION_COMBINED ADD PROJECTION section_daily_path_projection (
--     SELECT
--         api_key,
--         site,
--         path,
--         toStartOfDay(ts),
--         COUNT(*)
--     -- FROM SALUS_METRICS.SECTION_COMBINED
--     GROUP BY api_key, site, path, toStartOfDay(ts)
-- );
-- ALTER TABLE SALUS_METRICS.SECTION_COMBINED MATERIALIZE PROJECTION section_daily_path_projection;

-- ALTER TABLE SALUS_METRICS.SECTION_COMBINED DROP PROJECTION IF EXISTS section_daily_projection;
-- ALTER TABLE SALUS_METRICS.SECTION_COMBINED ADD PROJECTION section_daily_projection (
--     SELECT
--         api_key,
--         site,
--         toStartOfDay(ts),
--         COUNT(*),
--         uniqCombined(visitor)
--     -- FROM SALUS_METRICS.SECTION_COMBINED
--     GROUP BY api_key, site, toStartOfDay(ts)
-- );
-- ALTER TABLE SALUS_METRICS.SECTION_COMBINED MATERIALIZE PROJECTION section_daily_projection;

-- ALTER TABLE SALUS_METRICS.SECTION_COMBINED DROP INDEX IF EXISTS country_idx;
-- ALTER TABLE SALUS_METRICS.SECTION_COMBINED ADD INDEX country_idx country_code TYPE set(200) GRANULARITY 2;
-- ALTER TABLE SALUS_METRICS.SECTION_COMBINED MATERIALIZE INDEX country_idx;

ALTER TABLE SALUS_METRICS.SECTION_COMBINED DROP INDEX IF EXISTS city_idx;
ALTER TABLE SALUS_METRICS.SECTION_COMBINED ADD INDEX city_idx city TYPE bloom_filter GRANULARITY 1;
ALTER TABLE SALUS_METRICS.SECTION_COMBINED MATERIALIZE INDEX city_idx;

DROP TABLE IF EXISTS SALUS_METRICS.section_combined_mv;

CREATE MATERIALIZED VIEW SALUS_METRICS.section_combined_mv TO SALUS_METRICS.SECTION_COMBINED AS
SELECT
    SALUS_METRICS.SECTION_EVENT.api_key as api_key,
    SALUS_METRICS.SECTION_EVENT.site as site,
    toStartOfFiveMinutes(SALUS_METRICS.SECTION_EVENT.ts) as ts,
    SALUS_METRICS.SECTION_EVENT.path as path,
    sess.os as os,
    sess.browser as browser,
    sess.country_code as country_code,
    sess.city as city,
    murmurHash2_32(sess.parent) as visitor
FROM SALUS_METRICS.SECTION_EVENT
INNER JOIN (
    SELECT SALUS_METRICS.SESSION_EVENT.api_key,
        SALUS_METRICS.SESSION_EVENT.site,
        SALUS_METRICS.SESSION_EVENT.id,
        SALUS_METRICS.SESSION_EVENT.os,
        SALUS_METRICS.SESSION_EVENT.browser,
        SALUS_METRICS.SESSION_EVENT.country_code,
        SALUS_METRICS.SESSION_EVENT.city,
        SALUS_METRICS.SESSION_EVENT.parent
    FROM SALUS_METRICS.SESSION_EVENT
    WHERE SALUS_METRICS.SESSION_EVENT.id in (
        SELECT SALUS_METRICS.SECTION_EVENT.parent
        FROM SALUS_METRICS.SECTION_EVENT
    )
) AS sess
ON (SALUS_METRICS.SECTION_EVENT.api_key = sess.api_key
    AND SALUS_METRICS.SECTION_EVENT.site = sess.site
    AND SALUS_METRICS.SECTION_EVENT.parent = sess.id)
SETTINGS join_algorithm = 'full_sorting_merge'
;
