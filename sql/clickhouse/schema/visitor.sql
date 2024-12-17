CREATE TABLE SALUS_METRICS.VISITOR_EVENT (
    `api_key` LowCardinality (String) CODEC (ZSTD (1)),
    `site` LowCardinality (String) CODEC (ZSTD (1)),
    `id` UUID CODEC (ZSTD (1)),
    `ts` DateTime CODEC (Delta (4), ZSTD (1)),
    `attrs` Map (LowCardinality (String), String) CODEC (ZSTD (1))
) ENGINE = MergeTree
ORDER BY
    (api_key, site, ts);

CREATE MATERIALIZED VIEW SALUS_METRICS.visitor_event_mv TO SALUS_METRICS.VISITOR_EVENT AS
SELECT
    api_key,
    site,
    id,
    ts,
    attrs
FROM
    SALUS_METRICS.EVENT
WHERE
    event_type = 'Visitor'
    AND dictHas (
        'SALUS_METRICS.api_key_dictionary',
        (api_key, site)
    ) = 1;

CREATE TABLE SALUS_METRICS.VISITOR_TIMESERIES (
    `api_key` LowCardinality (String) CODEC (ZSTD (1)),
    `site` LowCardinality (String) CODEC (ZSTD (1)),
    `ts_bin` DateTime CODEC (Delta (4), ZSTD (1)),
    `num` UInt64 CODEC (ZSTD (1)),
) ENGINE = SummingMergeTree
ORDER BY
    (api_key, site, ts_bin);

CREATE MATERIALIZED VIEW SALUS_METRICS.visitor_timeseries_mv TO SALUS_METRICS.VISITOR_TIMESERIES AS
SELECT
    api_key,
    site,
    toStartOfFiveMinutes (ts) as ts_bin,
    count() as num
FROM
    SALUS_METRICS.VISITOR_EVENT
GROUP BY
    api_key,
    site,
    ts_bin
