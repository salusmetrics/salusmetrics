//! Ingest service and corresponding HTTP server that accepts incoming POST
//! requests from other sites that must already have been configured to be
//! on the CORS accept list for this particular setup.
//!
//! `bin/ingest_server.rs` can be run as a HTTP server and relies on axum
//! for all HTTP handling. Configuration strictly follows the 12 factor
//! approach with ENV variables used to specify all configuration options.
//!
//! All ENV variables are prefixed with `SALUS_INGEST_` and use the `conf`
//! crate for getting all configuration. The list of possible settings for
//! this app are as follows:
//! - `SALUS_INGEST_LAYER_COMPRESSION_DEFLATE` - OPTIONAL - values of `true` or `false` to
//!   enable or disable deflate compression. If neither this nor gzip are set,
//!   both default to true.
//! - `SALUS_INGEST_LAYER_COMPRESSION_GZIP` -OPTIONAL - values of `true` or `false` to
//!   enable or disable gzip compression. If neither this nor deflate are set
//!   then both will default to true.
//! - `SALUS_INGEST_LAYER_CORS_MAX_AGE_SECS` - OPTIONAL - integer value accepted
//!   to set the CORS max age. If not specified then the default of requiring
//!   an HTTP OPTIONS call for every CORS request will be used.
//! - `SALUS_INGEST_LAYER_CORS_ORIGINS` - REQUIRED - list of strings that specify
//!   the domains that are allowed to submite CORS requests to this server.
//! - `SALUS_INGEST_LAYER_TIMEOUT_MILLIS` - OPTIONAL - Integer accepted to
//!   specify the timeout for all requests. If no value is provided, default of
//!   30 seconds will be used.
//! - `SALUS_INGEST_LISTENER_IPV4` or `SALUS_INGEST_LISTENER_IPV6` - OPTIONAL -
//!   These are mutually exclusive and an error will be returned if both are set.
//!   The value determines the IP address on which the server will listen. If
//!   neither value is provided, then the IPv4 `0.0.0.0` will be used.
//! - `SALUS_INGEST_LISTENER_PORT` - REQUIRED - Integer value accepted to
//!   specify the port on which this server will listen and respond to requests
//! - `SALUS_INGEST_METRICSDB_DATABASE` - REQUIRED - Name of the metrics
//!   database in the specified Clickhouse instance in which data should be
//!   stored
//! - `SALUS_INGEST_METRICSDB_PASS` - REQUIRED - Password to log into the
//!   associated Clickhouse instance
//! - `SALUS_INGEST_METRICSDB_URL` - REQUIRED - URL for the associated
//!   Clickhouse instance
//! - `SALUS_INGEST_METRICSDB_USER` - REQUIRED - User on Clickhouse instance
//!   that should be used for recording data
//! - `SALUS_INGEST_TRACING` - OPTIONAL - string which must be a valid tracing
//!   subscriber directive. Defaults to `error` if no value is provided

pub mod domain;
pub mod http_api;
pub mod repositories;
pub mod services;
