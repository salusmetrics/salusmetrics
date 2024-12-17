CREATE TABLE SALUS_METRICS.SECTION_EVENT (
    `api_key` LowCardinality (String) CODEC (ZSTD (1)),
    `site` LowCardinality (String) CODEC (ZSTD (1)),
    `path` String CODEC (ZSTD (1)),
    `resource` String CODEC (ZSTD (1)),
    `query`  String CODEC (ZSTD (1)),
    `title` String CODEC (ZSTD (1)),
    `id` UUID CODEC (ZSTD (1)),
    `ts` DateTime CODEC(Delta(4), ZSTD(1)),
    `parent` UUID ALIAS attrs['parent'],
    `attrs` Map (LowCardinality (String), String) CODEC (ZSTD (1))
) ENGINE = MergeTree
ORDER BY
    (api_key, site, path, resource, ts)
TTL ts + INTERVAL 1 WEEK;

CREATE MATERIALIZED VIEW SALUS_METRICS.section_event_mv TO SALUS_METRICS.SECTION_EVENT AS
SELECT
    api_key,
    site,
    attrs['path'] as path,
    attrs['resource'] as resource,
    attrs['query'] as query,
    attrs['title'] as title,
    id,
    ts,
    attrs
FROM
    SALUS_METRICS.EVENT
WHERE
    event_type = 'Section'
    AND dictHas (
        'SALUS_METRICS.api_key_dictionary',
        (api_key, site)
    ) = 1
    AND attrs['parent'] > '';



CREATE TABLE SALUS_METRICS.SECTION_TIMESERIES (
    `api_key` LowCardinality (String) CODEC (ZSTD (1)),
    `site` LowCardinality (String) CODEC (ZSTD (1)),
    `path` String CODEC (ZSTD (1)),
    `resource` String CODEC (ZSTD (1)),
    `ts_bin` DateTime CODEC (Delta (4), ZSTD (1)),
    `num` UInt64 CODEC (ZSTD (1)),
) ENGINE = SummingMergeTree
ORDER BY
    (api_key, site, path, resource, ts_bin);

CREATE MATERIALIZED VIEW SALUS_METRICS.section_timeseries_mv TO SALUS_METRICS.SECTION_TIMESERIES AS
SELECT
    api_key,
    site,
    path,
    resource,
    toStartOfFiveMinutes (ts) as ts_bin,
    count() as num
FROM
    SALUS_METRICS.SECTION_EVENT
GROUP BY
    api_key,
    site,
    path,
    resource,
    ts_bin;
