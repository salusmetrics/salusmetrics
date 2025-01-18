use tower_http::compression::CompressionLayer;

/// `CompressionSettings` allows the setup of `tower-http` `CompressionLayer`
/// `gzip` and `deflate` are booleans that control those attributes respectively
#[derive(Debug, Clone)]
pub struct CompressionSettings {
    pub gzip: Option<bool>,
    pub deflate: Option<bool>,
}

impl From<&CompressionSettings> for CompressionLayer {
    fn from(value: &CompressionSettings) -> Self {
        CompressionLayer::new()
            .gzip(value.gzip.unwrap_or(true))
            .deflate(value.deflate.unwrap_or(true))
    }
}

impl Default for CompressionSettings {
    fn default() -> Self {
        Self {
            gzip: Some(true),
            deflate: Some(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use tower_http::compression::CompressionLayer;

    use super::*;

    #[test]
    fn test_compression_settings() {
        // Positive test case
        let test_settings = CompressionSettings::default();
        let _ = CompressionLayer::from(&test_settings);
    }
}
