use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Common function to set up tracing subscriber across services.
pub fn init_tracing_subscriber() {
    // Set environment variable RUST_LOG to specify log levels. i.e. RUST_LOG=error
    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| format!("{}=trace", env!("CARGO_CRATE_NAME")).into());

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(tracing_subscriber::fmt::layer().compact().with_target(true))
        .init();
}
