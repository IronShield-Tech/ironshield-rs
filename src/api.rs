//! # Error Handling enum and constants.
//! Copied from ironshield-api to avoid dependency

use thiserror::Error;
use std::time::Duration;

pub const  MAX_TIME_DIFF_MS:  i64 = 3 * 10000; // 3 * 10,000 milliseconds = 30 seconds
pub const      PUB_KEY_FAIL: &str = "Failed to load public key";
pub const      SIG_KEY_FAIL: &str = "Failed to load signing key";
pub const    SIGNATURE_FAIL: &str = "Signature verification failed";
pub const  INVALID_ENDPOINT: &str = "Endpoint must be a valid HTTPS URL";
pub const        CLOCK_SKEW: &str = "Request timestamp does not match the current time";
pub const    INVALID_PARAMS: &str = "Invalid challenge parameters";
pub const  INVALID_SOLUTION: &str = "Invalid solution provided for the challenge";

// Extended error types for projects that reference this API.
#[allow(dead_code)]
pub const     NETWORK_ERROR: &str = "Network request failed";

#[allow(dead_code)]
pub const     TIMEOUT_ERROR: &str = "Operation timed out";

#[allow(dead_code)]
pub const      CONFIG_ERROR: &str = "Invalid configuration";

#[allow(dead_code)]
pub const CHALLENGE_EXPIRED: &str = "Challenge has expired";

#[allow(dead_code)]
pub const    MAX_ITERATIONS: &str = "Maximum solving iterations reached without finding solution";

#[derive(Error, Debug)]
pub enum ErrorHandler {
    #[error("Invalid request format: {0}")]
    InvalidRequest(String),
    #[error("Processing failed: {0}")]
    ProcessingError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Internal server error")]
    #[allow(dead_code)]
    InternalError,

    // Extended error types for projects that reference this API.

    #[error("Network request failed: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("Operation timed out after {duration:?}")]
    #[allow(dead_code)]
    TimeoutError { duration: Duration },
    #[error("Configuration error: {0}")]
    #[allow(dead_code)]
    ConfigurationError(String),
    #[error("Challenge solving failed: {0}")]
    #[allow(dead_code)]
    ChallengeSolvingError(String),
    #[error("Challenge verification failed: {0}")]
    #[allow(dead_code)]
    ChallengeVerificationError(String),
    #[error("Authentication failed: {0}")]
    #[allow(dead_code)]
    AuthenticationError(String),
    #[error("Rate limit exceeded: {0}")]
    #[allow(dead_code)]
    RateLimitError(String),
    #[error("Resource not found: {0}")]
    #[allow(dead_code)]
    NotFoundError(String),
    #[error("Permission denied: {0}")]
    #[allow(dead_code)]
    PermissionError(String),
}

impl ErrorHandler {
    /// # Arguments
    /// * `error`: A `reqwest` network error.
    ///
    /// # Returns
    /// * `Self`: An `ErrorHandler::NetworkError` passed with the
    ///           argument provided to this function.
    #[allow(dead_code)]
    pub fn from_network_error(error: reqwest::Error) -> Self {
        ErrorHandler::NetworkError(error)
    }

    /// # Arguments
    /// * `message`: The duration of the request.
    ///
    /// # Returns
    /// * `Self`: An `ErrorHandler::TimeoutError` passed with the
    ///           argument provided to this function.
    #[allow(dead_code)]
    pub fn timeout(duration: Duration) -> Self {
        ErrorHandler::TimeoutError { duration }
    }

    /// # Arguments
    /// * `message`: The error message thrown on the event
    ///              configuration fails.
    ///
    /// # Returns
    /// * `Self`: An `ErrorHandler::ConfigurationError` passed
    ///           with the argument provided to this function.
    #[allow(dead_code)]
    pub fn config_error(message: impl Into<String>) -> Self {
        ErrorHandler::ConfigurationError(message.into())
    }

    /// # Arguments
    /// * `message`: The error message thrown on the event
    ///              solving a challenge fails.
    ///
    /// # Returns
    /// * `Self`: An `ErrorHandler::ChallengeSolvingError` passed with
    ///           argument provided to this function.
    #[allow(dead_code)]
    pub fn challenge_solving_error(message: impl Into<String>) -> Self {
        ErrorHandler::ChallengeSolvingError(message.into())
    }

    /// # Arguments
    /// * `message`: The error message thrown on the event
    ///              verification of a challenge fails.
    ///
    /// # Returns
    /// * `Self`: An `ErrorHandler::ChallengeVerificationError` passed
    ///           with the argument provided to this function.
    #[allow(dead_code)]
    pub fn challenge_verification_error(message: impl Into<String>) -> Self {
        ErrorHandler::ChallengeVerificationError(message.into())
    }

    /// # Arguments
    /// * `message`: The error message thrown on the event
    ///              authentication fails.
    ///
    /// # Returns
    /// * `Self`: An `ErrorHandler::AuthenticationError` passed with
    ///           the argument provided to this function.
    #[allow(dead_code)]
    pub fn authentication_error(message: impl Into<String>) -> Self {
        ErrorHandler::AuthenticationError(message.into())
    }

    /// # Arguments
    /// * `message`: The error message thrown on the event
    ///              a rate limit error occurs.
    ///
    /// # Returns
    /// * `Self`: An `ErrorHandler::RateLimitError` passed with
    ///           the argument provided to this function.
    #[allow(dead_code)]
    pub fn rate_limit_error(message: impl Into<String>) -> Self {
        ErrorHandler::RateLimitError(message.into())
    }

    /// # Arguments
    /// * `message`: The error message thrown on the event
    ///              a `404` or "not found" error occurs.
    ///
    /// # Returns
    /// * `Self`: An `ErrorHandler::NotFoundError` passed with
    ///           the argument provided to this function.
    #[allow(dead_code)]
    pub fn not_found_error(message: impl Into<String>) -> Self {
        ErrorHandler::NotFoundError(message.into())
    }

    /// # Arguments
    /// * `message`: The error message thrown on the event
    ///              that a permission error occurs.
    ///
    /// # Returns
    /// * `Self`: An `ErrorHandler::PermissionError` passed with
    ///           the argument provided to this function.
    #[allow(dead_code)]
    pub fn permission_error(message: impl Into<String>) -> Self {
        ErrorHandler::PermissionError(message.into())
    }
}



/// Type alias for function signatures.
pub type ResultHandler<T> = Result<T, ErrorHandler>; 