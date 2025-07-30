# Salus Metrics - Web and App Analytics for Regulated Industries

Every business needs the ability to understand how their products and tools are
being used. For most that now boils down to analytics packages that track how
web and mobile applications are used. The go-to solution for many years for most
businesses has been Google Analytics due to its price (free) but also the wide
support in the industry to integrate into many products and workflows. There
are businesses, however, which cannot use Google Analytics or which choose not
to out of the desire to keep their data from going to Google. Prime among these
businesses that _cannot_ use Goggle Analytics are entities which handle health
data that is protected by HIPAA and similar laws.

That's where tools like Salus Metrics Come in.

# Cost Effective In-House Analytics

Salus Metrics aims to provide the lowest cost to capture and report on web and
app analytics in an in-house package. There are three main strategies for
minimizing costs:

- Efficient storage & analysis: All data for Salus Metrics is stored in
  ClickHouse. This minimizes the storage cost and also provides excellent query
  performance for OLAP queries that are needed to generate reports.
- Separation of Ingest & Report: The architecture for Salus Metrics is designed
  so that ingestion of event data is separated from the reporting and display of
  the analytics. This allows each to scale up or down as needed, saving compute
  costs. Display and analysis of recorded data is done through Apache Superset,
  allowing organizations to use existing auth infrastructure for controlling
  access to potentially sensitive data.
- Consistent, Predictable Ingest Resource Usage: Using rust to provide
  consistent memory and CPU performance allows straightforward scaling as traffic
  goes up or down.

# Running Salus Metrics Ingest

The Salus Metrics ingest server requires that a ClickHouse database with the
appropriate schema has already been deployed. Schema can be found in
`sql/clickhouse/schema`

Once ClickHouse is running and has appropriate schema deployed, you can configure
the ingest server. All configuration for the ingest server is provided via ENV
variables, following the 12-factor application paradigm:

```sh
SALUS_INGEST_IP_SOURCE=ConnectInfo
SALUS_INGEST_LAYER_COMPRESSION_DEFLATE=true
SALUS_INGEST_LAYER_COMPRESSION_GZIP=true
SALUS_INGEST_LAYER_CORS_MAX_AGE_SECS=120
SALUS_INGEST_LAYER_CORS_ORIGINS=http://example.com http://www.example.com
SALUS_INGEST_LAYER_TIMEOUT_MILLIS=15000
SALUS_INGEST_LISTENER_IPV4=127.0.0.1
SALUS_INGEST_LISTENER_PORT=3000
SALUS_INGEST_METRICSDB_DATABASE=SALUS_METRICS
SALUS_INGEST_METRICSDB_PASS=****************
SALUS_INGEST_METRICSDB_URL=http://clickhouse.host.name:8123
SALUS_INGEST_METRICSDB_USER=********
SALUS_INGEST_TRACING_DIRECTIVE=trace
```

Only a subset of the above options is required, including all of the data about
ClickHouse as well as the CORS Origins. Depending on where and how you are
running the ingest server, you will have different requirements around the
listener port, listener IP (which supports IPV4 and IPV6) and the IP Source.
Assuming that the application is running behind a firewall or proxy, you will
need to use one of the values provided in the `axum-client-ip`
[crate](https://crates.io/crates/axum-client-ip).

This repo is structured as a workspace, so if you wish to run the ingest server
using cargo, you will need to specify it by name as follows:

```sh
cargo run --bin ingest_server
```
