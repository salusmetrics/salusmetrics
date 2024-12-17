CREATE TABLE SALUS_METRICS.API_KEY (
    `api_key` LowCardinality (String) CODEC (ZSTD (1)),
    `site` LowCardinality (String) CODEC (ZSTD (1)),
    `customer` LowCardinality (String) CODEC (ZSTD (1))
) ENGINE = ReplacingMergeTree PRIMARY KEY (api_key, site);

CREATE DICTIONARY SALUS_METRICS.api_key_dictionary (
    `api_key` String,
    `site` String,
    `customer` String
) PRIMARY KEY (api_key, site) SOURCE (CLICKHOUSE (TABLE 'API_KEY')) LAYOUT (COMPLEX_KEY_HASHED ()) LIFETIME (600);
