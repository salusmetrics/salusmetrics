DROP TABLE IF EXISTS SALUS_METRICS.SESSION_EVENT;

CREATE TABLE SALUS_METRICS.SESSION_EVENT (
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


DROP TABLE IF EXISTS SALUS_METRICS.session_event_mv;

CREATE MATERIALIZED VIEW SALUS_METRICS.session_event_mv TO SALUS_METRICS.SESSION_EVENT AS
SELECT
    api_key,
    site,
    id,
    ts,
    attrs,
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
    ) as browser_version,
    COALESCE(tupleElement (loc_tuple, 1), 'unknown') as country_code,
    COALESCE(tupleElement (loc_tuple, 2), 'unknown') as state,
    COALESCE(tupleElement (loc_tuple, 3), 'unknown') as city
FROM (
    SELECT
        api_key,
        site,
        id,
        ts,
        attrs,
        attrs['user_agent'] as user_agent,
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
        ) as browser_tuple,
        attrs['ipv4'] as ipv4,
        dictGetOrNull('SALUS_METRICS.dbip_city_ipv4_trie',
            ('country_code', 'state', 'city', 'latitude', 'longitude'), coalesce(toIPv4(ipv4), toIPv4(0))) as loc_tuple
    FROM SALUS_METRICS.EVENT
    WHERE
        event_type = 'Session'
        AND dictHas (
            'SALUS_METRICS.api_key_dictionary',
            (api_key, site)
        ) = 1
        AND attrs['parent'] > ''
)
ORDER BY
    (api_key, site, id);


-- DROP TABLE IF EXISTS SALUS_METRICS.SESSION_TIMESERIES;

-- CREATE TABLE SALUS_METRICS.SESSION_TIMESERIES (
--     `api_key` LowCardinality (String) CODEC (ZSTD (1)),
--     `site` LowCardinality (String) CODEC (ZSTD (1)),
--     `ts_bin` DateTime CODEC (Delta (4), ZSTD (1)),
--     `country_code` LowCardinality (String) CODEC (ZSTD (1)),
--     `os` AggregateFunction(sumMap, Tuple(Array(String), Array(UInt64))),
--     -- `browser` LowCardinality (String) CODEC (ZSTD (1)),
--     -- `city` String CODEC (ZSTD (1)),
--     `num` AggregateFunction(count, UInt64),
--     `uniq_visitors` AggregateFunction(uniq, UUID)
-- ) ENGINE = AggregatingMergeTree
-- ORDER BY
--     (api_key, site, ts_bin, country_code);


-- DROP TABLE IF EXISTS SALUS_METRICS.session_timeseries_mv;

-- CREATE MATERIALIZED VIEW SALUS_METRICS.session_timeseries_mv TO SALUS_METRICS.SESSION_TIMESERIES AS
-- SELECT
--     SALUS_METRICS.SESSION_EVENT.api_key AS api_key,
--     SALUS_METRICS.SESSION_EVENT.site AS site,
--     toStartOfHour(SALUS_METRICS.SESSION_EVENT.ts) as ts_bin,
--     SALUS_METRICS.SESSION_EVENT.country_code AS country_code,
--     sumMapState(tuple(array(SALUS_METRICS.SESSION_EVENT.os), array(toUInt64(1)))) AS os,
--     sumMapState(tuple(array(SALUS_METRICS.SESSION_EVENT.browser), array(toUInt64(1)))) AS browser,
--     -- SALUS_METRICS.SESSION_EVENT.city AS city,
--     countState() as num,
--     uniqState(SALUS_METRICS.SESSION_EVENT.parent) as uniq_visitors
-- FROM
--     SALUS_METRICS.SESSION_EVENT
-- INNER JOIN SALUS_METRICS.VISITOR_EVENT
--     ON (SALUS_METRICS.SESSION_EVENT.parent = SALUS_METRICS.VISITOR_EVENT.id
--         AND SALUS_METRICS.SESSION_EVENT.api_key = SALUS_METRICS.VISITOR_EVENT.api_key
--         AND SALUS_METRICS.SESSION_EVENT.site = SALUS_METRICS.VISITOR_EVENT.site)
-- GROUP BY
--     api_key,
--     site,
--     ts_bin,
--     country_code
--     ;
