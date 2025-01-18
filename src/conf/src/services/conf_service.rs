use crate::domain::service::configuration_service::ConfigurationService;

/// `ConfService<T>` is a generic implementation of the `ConfigurationService`
/// that can use any corresponding `ConfigurationRepository` to provide
/// configuration to apps
#[derive(Clone, Debug)]
pub struct ConfService<T>
where
    T: ConfigurationService + std::fmt::Debug,
{
    pub conf_repository: T,
}
