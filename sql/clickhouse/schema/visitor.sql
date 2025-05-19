DROP TABLE IF EXISTS SALUS_METRICS.VISITOR_EVENT;

CREATE TABLE SALUS_METRICS.VISITOR_EVENT (
    `api_key` LowCardinality (String) CODEC (ZSTD),
    `site` LowCardinality (String) CODEC (ZSTD),
    `id` UUID CODEC (ZSTD),
    `ts` DateTime CODEC (Delta, ZSTD),
    `attrs` Map (LowCardinality (String), String) CODEC (ZSTD)
) ENGINE = MergeTree
ORDER BY
    (api_key, site, id);

DROP TABLE IF EXISTS SALUS_METRICS.visitor_event_mv;

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

-- ---------------------------------------------------------------------------
-- ---------------------------------------------------------------------------
DROP TABLE IF EXISTS SALUS_METRICS.VISITOR_TIMESERIES;

CREATE TABLE SALUS_METRICS.VISITOR_TIMESERIES (
    `api_key` LowCardinality (String) CODEC (ZSTD),
    `site` LowCardinality (String) CODEC (ZSTD),
    `ts` DateTime CODEC (Delta, ZSTD),
    `visitors` AggregateFunction (sum, UInt32)
) ENGINE = AggregatingMergeTree
ORDER BY
    (api_key, site, ts);

DROP TABLE IF EXISTS SALUS_METRICS.visitor_timeseries_mv;

CREATE MATERIALIZED VIEW SALUS_METRICS.visitor_timeseries_mv TO SALUS_METRICS.VISITOR_TIMESERIES AS
SELECT
    api_key,
    site,
    toStartOfFiveMinutes (ts) as ts,
    sumState (toUInt32 (1)) as visitors
FROM
    SALUS_METRICS.VISITOR_EVENT
GROUP BY
    api_key,
    site,
    ts;
