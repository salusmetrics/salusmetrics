-- Want to use asynchronous insert to get best use of resources.
-- Also use query cache for best performance SETTINGS use_query_cache = true
CREATE TABLE SALUS_METRICS.EVENT (
    `api_key` LowCardinality (String),
    `site` LowCardinality (String),
    `event_type` Enum8 (
        'Visitor' = 1,
        'Session' = 2,
        'Section' = 4,
        'Click' = 5
    ),
    `id` UUID,
    `ts` DateTime DEFAULT UUIDv7ToDateTime (id),
    `attrs` Map (LowCardinality (String), String),
) ENGINE = Null;
