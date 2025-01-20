use crate::{
    domain::service::configuration_service::ConfigurationServiceError,
    repositories::env::env_repository::EnvRepository, services::conf_service::ConfService,
};

/// Convenience function to get a `ConfService<EnvRepository>` for a given
/// `app_prefix`
pub fn env_conf(
    app_prefix: impl AsRef<str>,
) -> Result<ConfService<EnvRepository>, ConfigurationServiceError> {
    let repo =
        EnvRepository::try_new(app_prefix).map_err(|_| ConfigurationServiceError::Repository)?;
    Ok(ConfService::new(repo))
}
