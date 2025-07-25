use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    /// API-specific errors with status
    /// code and message from the server.
    #[error("API error ({status}): {message}")]
    Api {
        /// HTTP status code returned by the API.
        status:  u16,
        /// Error message from the API response.
        message: String
    },
    /// Configuration-related errors
    /// (invalid settings, missing files, etc.).
    #[error("Configuration error: {0}")]
    Config(String),
    /// Challenge processing errors
    /// (solving, verification, etc.).
    #[error("Challenge processing error: {0}")]
    Challenge(String),
    /// File system and I/O errors.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Network communication errors from the HTTP client.
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    /// JSON parsing and serialization errors.
    #[error("Parsing error: {0}")]
    Parse(#[from] serde_json::Error),
    /// TOML configuration file parsing errors.
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),
}

impl CliError {
    /// # Arguments
    /// * `status`:  The HTTP status code from the API
    ///              response.
    /// * `message`: The error message that corresponds
    ///              to the API error.
    ///
    /// # Returns
    /// * `Self`: A new `CliError::Api` variant.
    pub fn api_error(
        status: u16,
        message: impl Into<String>
    ) -> Self {
        Self::Api { status, message: message.into() }
    }

    /// # Arguments
    /// * `message`: The error message that corresponds
    ///              to the config error.
    ///
    /// # Returns
    /// * `Self`: A new `CliError::Config` variant.
    pub fn config_error(
        message: impl Into<String>
    ) -> Self {
        Self::Config(message.into())
    }

    /// # Arguments
    /// * `message`: The error message that corresponds
    ///              to the challenge error.
    ///
    /// # Returns
    /// * `Self`: A new `CliError::Challenge` variant.
    pub fn challenge_error(
        message: impl Into<String>
    ) -> Self {
        Self::Challenge(message.into())
    }
} 