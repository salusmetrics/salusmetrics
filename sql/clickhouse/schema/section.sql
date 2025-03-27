DROP TABLE IF EXISTS SALUS_METRICS.SECTION_EVENT;

CREATE TABLE SALUS_METRICS.SECTION_EVENT (
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
    (api_key, site, id)
-- TTL ts + INTERVAL 1 WEEK
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



-- DROP TABLE IF EXISTS SALUS_METRICS.SECTION_TIMESERIES;

-- CREATE TABLE SALUS_METRICS.SECTION_TIMESERIES (
--     `api_key` LowCardinality (String) CODEC (ZSTD (1)),
--     `site` LowCardinality (String) CODEC (ZSTD (1)),
--     `path` String CODEC (ZSTD (1)),
--     `os` LowCardinality (String) CODEC (ZSTD (1)),
--     `browser` LowCardinality (String) CODEC (ZSTD (1)),
--     `country_code` LowCardinality (String) CODEC (ZSTD (1)),
--     `city` String CODEC (ZSTD (1)),
--     `ts_bin` DateTime CODEC (Delta (4), ZSTD (1)),
--     `num` AggregateFunction(count, UInt64),
--     `uniq_visitors` AggregateFunction(uniq, UUID)
-- ) ENGINE = AggregatingMergeTree
-- ORDER BY
--     (api_key, site, ts_bin, path
--     , os, browser, country_code, city
--     );


-- DROP TABLE IF EXISTS SALUS_METRICS.section_timeseries_mv;

-- CREATE MATERIALIZED VIEW SALUS_METRICS.section_timeseries_mv TO SALUS_METRICS.SECTION_TIMESERIES AS
-- SELECT
--     SALUS_METRICS.SECTION_EVENT.api_key as api_key,
--     SALUS_METRICS.SECTION_EVENT.site as site,
--     SALUS_METRICS.SECTION_EVENT.path as path,
--     sess.os as os,
--     sess.browser as browser,
--     sess.country_code as country_code,
--     sess.city as city,
--     toStartOfHour(SALUS_METRICS.SECTION_EVENT.ts) as ts_bin,
--     countState() as num,
--     uniqState(sess.parent) as uniq_visitors
-- FROM SALUS_METRICS.SECTION_EVENT
-- -- INNER JOIN SALUS_METRICS.SESSION_EVENT
-- --     ON (SALUS_METRICS.SECTION_EVENT.api_key = SALUS_METRICS.SESSION_EVENT.api_key
-- --         AND SALUS_METRICS.SECTION_EVENT.site = SALUS_METRICS.SESSION_EVENT.site
-- --         AND SALUS_METRICS.SECTION_EVENT.parent = SALUS_METRICS.SESSION_EVENT.id)
-- INNER JOIN (
--     SELECT SALUS_METRICS.SESSION_EVENT.api_key,
--         SALUS_METRICS.SESSION_EVENT.site,
--         SALUS_METRICS.SESSION_EVENT.id,
--         SALUS_METRICS.SESSION_EVENT.os,
--         SALUS_METRICS.SESSION_EVENT.browser,
--         SALUS_METRICS.SESSION_EVENT.country_code,
--         SALUS_METRICS.SESSION_EVENT.city,
--         SALUS_METRICS.SESSION_EVENT.parent
--     FROM SALUS_METRICS.SESSION_EVENT
--     WHERE SALUS_METRICS.SESSION_EVENT.id in (
--         SELECT SALUS_METRICS.SECTION_EVENT.parent
--         FROM SALUS_METRICS.SECTION_EVENT
--     )
-- ) AS sess
-- ON (SALUS_METRICS.SECTION_EVENT.api_key = sess.api_key
--     AND SALUS_METRICS.SECTION_EVENT.site = sess.site
--     AND SALUS_METRICS.SECTION_EVENT.parent = sess.id)
-- GROUP BY
--     SALUS_METRICS.SECTION_EVENT.api_key,
--     SALUS_METRICS.SECTION_EVENT.site,
--     ts_bin,
--     SALUS_METRICS.SECTION_EVENT.path,
--     sess.os,
--     sess.browser,
--     sess.country_code,
--     sess.city
-- SETTINGS join_algorithm = 'full_sorting_merge'
--     ;

-- ---------------------------------------------------------------------------
-- ---------------------------------------------------------------------------

DROP TABLE IF EXISTS SALUS_METRICS.SECTION_COMBINED;

CREATE TABLE SALUS_METRICS.SECTION_COMBINED (
    `api_key` LowCardinality (String) CODEC (ZSTD (1)),
    `site` LowCardinality (String) CODEC (ZSTD (1)),
    `ts` DateTime CODEC (Delta (4), ZSTD (1)),
    `path` String CODEC (ZSTD (1)),
    `os` LowCardinality (String) CODEC (ZSTD (1)),
    `browser` LowCardinality (String) CODEC (ZSTD (1)),
    `country_code` LowCardinality (String) CODEC (ZSTD (1)),
    `city` String CODEC (ZSTD (1)),
    `visitor` UUID CODEC (ZSTD (1))
) ENGINE = MergeTree
ORDER BY
    (api_key, site, ts);

ALTER TABLE SALUS_METRICS.SECTION_COMBINED ADD PROJECTION section_path_projection (
    SELECT *
    ORDER BY (api_key, site, path, ts)
);
ALTER TABLE SALUS_METRICS.SECTION_COMBINED MATERIALIZE PROJECTION section_path_projection;


-- DROP TABLE IF EXISTS SALUS_METRICS.section_combined_mv;

-- CREATE MATERIALIZED VIEW SALUS_METRICS.section_combined_mv TO SALUS_METRICS.SECTION_COMBINED AS
-- SELECT
--     SALUS_METRICS.SECTION_EVENT.api_key as api_key,
--     SALUS_METRICS.SECTION_EVENT.site as site,
--     SALUS_METRICS.SECTION_EVENT.ts as ts,
--     SALUS_METRICS.SECTION_EVENT.path as path,
--     SALUS_METRICS.SESSION_EVENT.os as os,
--     SALUS_METRICS.SESSION_EVENT.browser as browser,
--     SALUS_METRICS.SESSION_EVENT.country_code as country_code,
--     SALUS_METRICS.SESSION_EVENT.city as city,
--     SALUS_METRICS.SESSION_EVENT.parent as visitor
-- FROM SALUS_METRICS.SECTION_EVENT
-- JOIN SALUS_METRICS.SESSION_EVENT
-- ON (SALUS_METRICS.SECTION_EVENT.api_key = SALUS_METRICS.SESSION_EVENT.api_key
--     AND SALUS_METRICS.SECTION_EVENT.site = SALUS_METRICS.SESSION_EVENT.site
--     AND SALUS_METRICS.SECTION_EVENT.parent = SALUS_METRICS.SESSION_EVENT.id)
-- SETTINGS join_algorithm = 'full_sorting_merge'
-- ;

-- DROP TABLE IF EXISTS SALUS_METRICS.section_combined_mv;

-- CREATE MATERIALIZED VIEW SALUS_METRICS.section_combined_mv TO SALUS_METRICS.SECTION_COMBINED AS
-- SELECT
--     SALUS_METRICS.SECTION_EVENT.api_key as api_key,
--     SALUS_METRICS.SECTION_EVENT.site as site,
--     SALUS_METRICS.SECTION_EVENT.ts as ts,
--     SALUS_METRICS.SECTION_EVENT.path as path,
--     sess.os as os,
--     sess.browser as browser,
--     sess.country_code as country_code,
--     sess.city as city,
--     sess.parent as visitor
-- FROM (
--     SELECT SALUS_METRICS.SESSION_EVENT.api_key as api_key,
--         SALUS_METRICS.SESSION_EVENT.site as site,
--         SALUS_METRICS.SESSION_EVENT.id as id,
--         SALUS_METRICS.SESSION_EVENT.os as os,
--         SALUS_METRICS.SESSION_EVENT.browser as browser,
--         SALUS_METRICS.SESSION_EVENT.country_code as country_code,
--         SALUS_METRICS.SESSION_EVENT.city as city,
--         SALUS_METRICS.SESSION_EVENT.parent
--     FROM SALUS_METRICS.SESSION_EVENT
--     WHERE SALUS_METRICS.SESSION_EVENT.id in (SELECT parent FROM SALUS_METRICS.SECTION_EVENT)) as sess
-- INNER JOIN SALUS_METRICS.SECTION_EVENT
-- ON (SALUS_METRICS.SECTION_EVENT.api_key = sess.api_key
--     AND SALUS_METRICS.SECTION_EVENT.site = sess.site
--     AND SALUS_METRICS.SECTION_EVENT.parent = sess.id)
-- SETTINGS join_algorithm = 'full_sorting_merge'
-- ;


DROP TABLE IF EXISTS SALUS_METRICS.section_combined_mv;

CREATE MATERIALIZED VIEW SALUS_METRICS.section_combined_mv TO SALUS_METRICS.SECTION_COMBINED AS
SELECT
    SALUS_METRICS.SECTION_EVENT.api_key as api_key,
    SALUS_METRICS.SECTION_EVENT.site as site,
    SALUS_METRICS.SECTION_EVENT.ts as ts,
    SALUS_METRICS.SECTION_EVENT.path as path,
    sess.os as os,
    sess.browser as browser,
    sess.country_code as country_code,
    sess.city as city,
    sess.parent as visitor
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
