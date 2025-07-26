use serde::{
    Deserialize,
    Serialize
};

use crate::USER_AGENT;

use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub api_base_url: String,
    pub num_threads:  Option<usize>,
    #[serde(with = "duration_serde")]
    pub timeout:      Duration,
    pub user_agent:   String,
    pub verbose:      bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            api_base_url: "https://api.ironshield.cloud".to_string(),
            num_threads:  None,
            timeout:      Duration::from_secs(30),
            user_agent:   USER_AGENT.to_string(),
            verbose:      false,
        }
    }
}

impl ClientConfig {
    /// Creates a development configuration.
    ///
    /// # Returns
    /// `Self`: A `ClientConfig` instance optimized for development use.
    ///
    /// # Example
    /// ```
    /// use ironshield::ClientConfig;
    ///
    /// let dev_config = ClientConfig::development();
    /// assert!(dev_config.verbose);
    /// ```
    pub fn development() -> Self {
        Self {
            api_base_url: "https://dev-api.ironshield.cloud".to_string(),
            num_threads:  Some(1),
            timeout:      Duration::from_secs(60),
            user_agent:   format!("{}-dev", USER_AGENT),
            verbose:      true,
        }
    }

    /// Creates a testing configuration for use with a locally run API.
    /// Made for port 3000.
    ///
    /// # Returns
    /// `Self`: A `ClientConfig` instance optimized for testing scenarios.
    ///
    /// # Example
    /// ```
    /// use ironshield::ClientConfig;
    ///
    /// let test_config = ClientConfig::testing();
    /// assert_eq!(test_config.api_base_url, "http://localhost:3000");
    /// ```
    pub fn testing() -> Self {
        Self {
            api_base_url: "http://localhost:3000".to_string(),
            num_threads:  Some(1),
            timeout:      Duration::from_secs(5),
            user_agent:   format!("{}-test", USER_AGENT),
            verbose:      false,
        }
    }

    /// Validates the current configuration, ensuring all values are within acceptable ranges.
    ///
    /// # Returns
    /// * `Result<(), ErrorHandler>`: Success indication or validation error.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The API base URL is empty or invalid
    /// - The timeout is zero or negative
    /// - The number of threads is zero
    /// - The user agent string is empty
    ///
    /// # Example
    /// ```
    /// use ironshield::ClientConfig;
    ///
    /// let config = ClientConfig::default();
    /// assert!(config.validate().is_ok());
    /// ```
    #[cfg(feature = "toml")]
    pub fn validate(&self) -> Result<(), ErrorHandler> {
        if self.api_base_url.is_empty() {
            return Err(ErrorHandler::config_error(
                "API base URL cannot be empty".to_string()
            ));
        }

        if !self.api_base_url.starts_with("https://") {
            return Err(ErrorHandler::config_error(
                INVALID_ENDPOINT
            ));
        }

        if self.timeout.is_zero() {
            return Err(ErrorHandler::config_error(
                "Timeout must be greater than zero".to_string()
            ));
        }

        if let Some(threads) = self.num_threads {
            if threads == 0 {
                return Err(ErrorHandler::config_error(
                    "Number of threads must be greater than zero".to_string()
                ));
            }
        }

        if self.user_agent.is_empty() {
            return Err(ErrorHandler::config_error(
                "User agent cannot be empty".to_string()
            ));
        }

        Ok(())
    }

    /// Loads a configuration file from a TOML file,
    /// falling back to defaults if it is not present.
    ///
    /// # Arguments
    /// * `path`: The path to the TOML configuration file.
    ///
    /// # Returns
    /// * `Result<Self, CliError>`: containing the loaded
    ///                             configuration, or an
    ///                             error if parsing fails.
    ///
    /// # Examples
    /// ```
    /// // Load from the default location.
    /// use ironshield::ClientConfig;
    /// 
    /// let config = ClientConfig::from_file("ironshield.toml")?;
    ///
    /// // Load from a custom location.
    /// let config = ClientConfig::from_file("/etc/ironshield/config.toml")?;
    /// ```
    #[cfg(feature = "toml")]
    pub fn from_file(path: &str) -> Result<ClientConfig, ErrorHandler> {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let config: ClientConfig = toml::from_str(&content)
                    .map_err(|e| ErrorHandler::config_error(
                        format!("Failed to parse TOML config file '{}': {}", path, e)
                    ))?;

                config.validate()
                      .map_err(|e| ErrorHandler::config_error(
                          format!("Configuration validation failed: {}", e)
                      ))?;

                Ok(config)
            }
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    eprintln!("Config file '{}' not found, using default configuration.", path);
                    Ok(ClientConfig::default())
                } else {
                    Err(ErrorHandler::Io(err))
                }
            }
        }
    }

    /// Saves the current configuration to a TOML file.
    ///
    /// # Arguments
    /// * `path`: Path to the configuration file save location.
    ///
    /// # Returns
    /// * `Result<(), ErrorHandler>`: Success indication or error.
    ///
    /// # Example
    /// ```
    /// use ironshield::ClientConfig;
    ///
    /// let config = ClientConfig::default();
    /// config.save_to_file("ironshield.toml")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[cfg(feature = "toml")]
    pub fn save_to_file(&self, path: &str) -> Result<(), ErrorHandler> {
        self.validate()?;
        
        let content = toml::to_string_pretty(self)
            .map_err(|e| ErrorHandler::config_error(
                format!("Failed to serialize config to TOML: {}", e)
            ))?;
        
        std::fs::write(path, content)
            .map_err(|e| ErrorHandler::Io(e))?;

        Ok(())
    }

    /// # Arguments
    /// * `url`: The new API base URL.
    ///
    /// # Returns
    /// * `Result<&mut Self, ErrorHandler>`: Mutable reference for method chaining or error.
    ///
    /// # Example
    /// ```
    /// use ironshield::ClientConfig;
    ///
    /// let mut config = ClientConfig::default();
    /// config.set_api_base_url("https://custom-api.example.com")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[cfg(feature = "toml")]
    pub fn set_api_base_url(&mut self, url: &str) -> Result<&mut Self, ErrorHandler> {
        if url.is_empty() {
            return Err(ErrorHandler::config_error(
                "API base URL cannot be empty".to_string()
            ));
        }

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(ErrorHandler::config_error(
                "API base URL must start with http:// or https://".to_string()
            ));
        }

        self.api_base_url = url.to_string();
        Ok(self)
    }

    /// # Arguments
    /// * `timeout`: The new timeout duration.
    ///
    /// # Returns
    /// * `Result<&mut Self, ErrorHandler>`: Mutable reference for method
    ///                                      chaining or error.
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    /// use ironshield::ClientConfig;
    ///
    /// let mut config = ClientConfig::default();
    /// config.set_timeout(Duration::from_secs(45))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[cfg(feature = "toml")]
    pub fn set_timeout(&mut self, timeout: Duration) -> Result<&mut Self, ErrorHandler> {
        if timeout.is_zero() {
            return Err(ErrorHandler::config_error(
                "Timeout must be greater than zero".to_string()
            ));
        }

        self.timeout = timeout;
        Ok(self)
    }

    /// Sets the number of threads after validation.
    ///
    /// # Arguments
    /// * `threads`: The number of threads to use, or None for
    ///              auto-detection.
    ///
    /// # Returns
    /// * `Result<&mut Self, ErrorHandler>`: Mutable reference for
    ///                                      method chaining or error.
    ///
    /// # Example
    /// ```
    /// use ironshield::ClientConfig;
    ///
    /// let mut config = ClientConfig::default();
    /// config.set_num_threads(Some(4))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[cfg(feature = "toml")]
    pub fn set_num_threads(&mut self, threads: Option<usize>) -> Result<&mut Self, ErrorHandler> {
        if let Some(thread_count) = threads {
            if thread_count == 0 {
                return Err(ErrorHandler::config_error(
                    "Number of threads must be greater than zero".to_string()
                ));
            }
        }

        self.num_threads = threads;
        Ok(self)
    }

    /// # Arguments
    /// * `verbose`: Whether to enable verbose logging.
    ///
    /// # Returns
    /// * `&mut Self`: Mutable reference for method chaining.
    ///
    /// # Example
    /// ```
    /// use ironshield::ClientConfig;
    ///
    /// let mut config = ClientConfig::default();
    /// config.set_verbose(true);
    /// assert!(config.verbose);
    /// ```
    pub fn set_verbose(&mut self, verbose: bool) -> &mut Self {
        self.verbose = verbose;
        self
    }

    /// # Arguments
    /// * `user_agent`: The new user agent string.
    ///
    /// # Returns
    /// * `Result<&mut Self, ErrorHandler>`: Mutable reference for method chaining or error.
    ///
    /// # Example
    /// ```
    /// use ironshield::ClientConfig;
    ///
    /// let mut config = ClientConfig::default();
    /// config.set_user_agent("whateva/1.0")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[cfg(feature = "toml")]
    pub fn set_user_agent(&mut self, user_agent: &str) -> Result<&mut Self, ErrorHandler> {
        if user_agent.is_empty() {
            return Err(ErrorHandler::config_error(
                "User agent cannot be empty".to_string()
            ));
        }

        self.user_agent = user_agent.to_string();
        Ok(self)
    }
}

/// Custom serialization/deserialization for `Duration` fields.
///
/// Provides serde support for `Duration` fields,
/// serializes them as seconds (u64) in TOML files
/// for human readability while maintaining type safety.
mod duration_serde {
    use serde::{
        Deserialize,
        Deserializer,
        Serializer
    };
    use std::time::Duration;

    /// Serializes a `Duration` as seconds.
    ///
    /// # Arguments
    /// * `duration`:   Duration to serialize.
    /// * `serializer`: The serde serializer.
    ///
    /// # Returns
    /// * `Result<S::Ok, S::Error>`: The serialized duration as an
    ///                              `u64` representing seconds on
    ///                              success, or a serialization
    ///                              error on failure.
    ///
    /// # Type Parameters
    /// * `S`: The serializer type that implements the `Serializer`
    ///        trait.
    pub fn serialize<S>(
        duration: &Duration,
        serializer: S
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    /// Deserializes a duration from seconds.
    ///
    /// # Arguments
    /// * `deserializer`: The serde deserializer.
    ///
    /// # Returns
    /// * `Result<Duration, D::Error>`: A `Duration` constructed
    ///                                 from the deserialized seconds
    ///                                 value on success, or a
    ///                                 deserialization error if the
    ///                                 operation fails.
    ///
    /// # Type Parameters
    /// * `D`: The deserializer type that implements the `Deserializer`
    ///        trait.
    pub fn deserialize<'de, D>(
        deserializer: D
    ) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    #[cfg(feature = "toml")]
    fn test_default_config_is_valid() {
        let config = ClientConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    #[cfg(feature = "toml")]
    fn test_config_validation_invalid_url() {
        let mut config = ClientConfig::default();
        config.api_base_url = "http://insecure.example.com".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    #[cfg(feature = "toml")]
    fn test_config_validation_invalid_timeout() {
        let mut config = ClientConfig::default();
        config.timeout = Duration::from_secs(0);
        assert!(config.validate().is_err());
    }

    #[test]
    #[cfg(feature = "toml")]
    fn test_config_validation_invalid_threads() {
        let mut config = ClientConfig::default();
        config.num_threads = Some(0);
        assert!(config.validate().is_err());
    }
}