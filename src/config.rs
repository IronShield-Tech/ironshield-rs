use serde::{
    Deserialize,
    Serialize
};

use crate::error::{ErrorHandler, INVALID_ENDPOINT};

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
            user_agent:   crate::constant::USER_AGENT.to_string(),
            verbose:      false,
        }
    }
}

#[allow(dead_code)]
impl ClientConfig {
    /// Loads a configuration file from a TOML file,
    /// falling back to defaults if it is not present.
    ///
    /// # Arguments
    /// * `path`: The path to the TOML configuration file.
    ///
    /// # Returns
    /// * `Result<Self, ErrorHandler>`: containing the loaded
    ///                                 configuration, or an
    ///                                 error if parsing fails.
    ///
    /// # Examples
    /// ```no_run
    /// use ironshield::ClientConfig;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Load from default location.
    /// let config = ClientConfig::from_file("ironshield.toml")?;
    ///
    /// // Load from custom location.
    /// let config = ClientConfig::from_file("/etc/ironshield/config.toml")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_file(path: &str) -> Result<Self, ErrorHandler> {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let config: Self = toml::from_str(&content)?;

                config.validate()?;

                Ok(config)
            }
            Err(err) => { // File doesn't exist, use the default configuration.
                if err.kind() == std::io::ErrorKind::NotFound {
                    eprintln!("Config file '{}' not found, using default configuration.", path);
                    Ok(Self::default())
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
    /// * `Result<(), ErrorHandler
    /// >`: Indication of success or failure.
    ///
    /// # Examples
    /// ```no_run
    /// use ironshield::ClientConfig;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ClientConfig::default();
    /// config.save_to_file("ironshield.toml")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn save_to_file(&self, path: &str) -> Result<(), ErrorHandler> {
        self.validate()?;

        let content = toml::to_string_pretty(self)
            .map_err(|e| ErrorHandler::config_error(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(path, content)?;

        Ok(())
    }

    /// Validates the configuration.
    ///
    /// # Returns
    /// * `Result<(), ErrorHandler
    /// >`: Indication of success or failure.
    fn validate(&self) -> Result<(), ErrorHandler
    > {
        let timeout_secs = self.timeout.as_secs();

        if !self.api_base_url.starts_with("https://") {
            return Err(ErrorHandler
            ::config_error(
                INVALID_ENDPOINT
            ))
        }

        if timeout_secs < 1 || timeout_secs > 600 {
            return Err(ErrorHandler
            ::config_error(
                "Timeout must be between 1 seconds and 10 minutes."
            ))
        }

        if let Some(threads) = self.num_threads {
            if threads == 0 {
                return Err(ErrorHandler
                ::config_error(
                    "Thread count must be greater than 0."
                ))
            }
        }

        Ok(())
    }

    pub fn development() -> Self {
        Self {
            api_base_url: "https://localhost:3000".to_string(),
            num_threads:  Some(2), // Use limited threading for development.
            timeout:      Duration::from_secs(10),
            user_agent:   crate::constant::USER_AGENT.to_string(),
            verbose:      true,
        }
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
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config_is_valid() {
        let config = ClientConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_invalid_url() {
        let mut config = ClientConfig::default();
        config.api_base_url = "http://insecure.example.com".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_invalid_timeout() {
        let mut config = ClientConfig::default();
        config.timeout = Duration::from_secs(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_invalid_threads() {
        let mut config = ClientConfig::default();
        config.num_threads = Some(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_roundtrip() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_config.toml");
        let file_path_str = file_path.to_str().unwrap();

        let original_config = ClientConfig::development();
        original_config.save_to_file(file_path_str).unwrap();

        let loaded_config = ClientConfig::from_file(file_path_str).unwrap();

        assert_eq!(original_config.api_base_url, loaded_config.api_base_url);
        assert_eq!(original_config.timeout, loaded_config.timeout);
        assert_eq!(original_config.verbose, loaded_config.verbose);
        assert_eq!(original_config.num_threads, loaded_config.num_threads);
    }

    #[test]
    fn test_config_missing_file_uses_default() {
        let result = ClientConfig::from_file("nonexistent_file.toml");
        assert!(result.is_ok());

        let config = result.unwrap();
        let default_config = ClientConfig::default();
        assert_eq!(config.api_base_url, default_config.api_base_url);
    }
}