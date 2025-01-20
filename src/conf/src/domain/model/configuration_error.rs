use thiserror::Error;

/// `ConfigurationError` represents the domain errors that can arise
/// when a configuration model is applying domain rules. This includes the
/// possibility of converting from the domain structs into a desired end
/// form
#[derive(Clone, Error, Debug, PartialEq, Eq)]
pub enum ConfigurationError {
    /// `Invalid` arises when the configuration specified for the
    /// request type is not valid for the domain
    #[error("Configuration for this type was not valid")]
    Invalid,
    /// `Parse` indicates that some aspect of the configuration could not be
    /// parsed from the data source
    #[error("Could not parse configuration")]
    Parse,
}
