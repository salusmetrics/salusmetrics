DROP TABLE IF EXISTS SALUS_METRICS.SESSION_EVENT;

CREATE TABLE SALUS_METRICS.SESSION_EVENT (
    `api_key` LowCardinality (String) CODEC (ZSTD (1)),
    `site` LowCardinality (String) CODEC (ZSTD (1)),
    `id` UUID CODEC (ZSTD (1)),
    `ts` DateTime CODEC(Delta(4), ZSTD(1)),
    `parent` UUID ALIAS attrs['parent'],
    `user_agent` String ALIAS attrs['user_agent'],
    `ipv4` Nullable(IPv4) ALIAS attrs['ipv4'],
    `ipv6` Nullable(IPv6) ALIAS attrs['ipv6'],
    `attrs` Map (LowCardinality (String), String) CODEC (ZSTD (1))
) ENGINE = MergeTree
ORDER BY
    (api_key, site, ts);

DROP TABLE IF EXISTS SALUS_METRICS.session_event_mv;

CREATE MATERIALIZED VIEW SALUS_METRICS.session_event_mv TO SALUS_METRICS.SESSION_EVENT AS
SELECT
    api_key,
    site,
    id,
    ts,
    attrs
FROM
    SALUS_METRICS.EVENT
WHERE
    event_type = 'Session'
    AND dictHas (
        'SALUS_METRICS.api_key_dictionary',
        (api_key, site)
    ) = 1
    AND attrs['parent'] > '';


CREATE TABLE SALUS_METRICS.SESSION_TIMESERIES (
    `api_key` LowCardinality (String) CODEC (ZSTD (1)),
    `site` LowCardinality (String) CODEC (ZSTD (1)),
    `ts_bin` DateTime CODEC (Delta (4), ZSTD (1)),
    `num` UInt64 CODEC (ZSTD (1)),
) ENGINE = SummingMergeTree
ORDER BY
    (api_key, site, ts_bin);

CREATE MATERIALIZED VIEW SALUS_METRICS.session_timeseries_mv TO SALUS_METRICS.SESSION_TIMESERIES AS
SELECT
    api_key,
    site,
    toStartOfFiveMinutes (ts) as ts_bin,
    count() as num
FROM
    SALUS_METRICS.SESSION_EVENT
GROUP BY
    api_key,
    site,
    ts_bin;
