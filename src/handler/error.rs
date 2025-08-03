//! # Error Handling enum and constants.

use axum::{
    Json,
    http::StatusCode,
    response::{
        IntoResponse,
        Response
    },
};
use thiserror::Error;

use std::time::Duration;

/// Error information containing both message and HTTP status code
#[derive(Debug, Clone)]
pub struct ErrorInfo {
    pub message: &'static str,
    pub status_code: u16,
}

impl ErrorInfo {
    /// Get the message as a const function 
    pub const fn message(&self) -> &'static str {
        self.message
    }
    
    /// Get the status code as a const function
    pub const fn status_code(&self) -> u16 {
        self.status_code
    }
}

impl std::fmt::Display for ErrorInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

// HTTP Status Code Constants
pub const STATUS_OK: u16 = 200;
pub const STATUS_OK_MSG: &str = "Served response successfully.";
pub const STATUS_BAD_REQUEST: u16 = 400;
pub const STATUS_UNAUTHORIZED: u16 = 401;
pub const STATUS_FORBIDDEN: u16 = 403;
pub const STATUS_NOT_FOUND: u16 = 404;
pub const STATUS_GONE: u16 = 410;
pub const STATUS_UNPROCESSABLE_ENTITY: u16 = 422;
pub const STATUS_INTERNAL_SERVER_ERROR: u16 = 500;

// Error message constants (for utoipa descriptions)
pub const CLOCK_SKEW_MSG: &str = "Request timestamp does not match the current time";
pub const INVALID_ENDPOINT_MSG: &str = "Endpoint must be a valid HTTPS URL";
pub const INVALID_PARAMS_MSG: &str = "Invalid challenge parameters";
pub const INVALID_SOLUTION_MSG: &str = "Invalid solution provided for the challenge";
pub const CHALLENGE_EXPIRED_MSG: &str = "Challenge has expired";
pub const PUB_KEY_FAIL_MSG: &str = "Failed to load public key";
pub const SIG_KEY_FAIL_MSG: &str = "Failed to load signing key";
pub const SIGNATURE_FAIL_MSG: &str = "Signature verification failed";

// Error definitions with semantically correct status codes
pub const CLOCK_SKEW: ErrorInfo = ErrorInfo {
    message: CLOCK_SKEW_MSG,
    status_code: STATUS_BAD_REQUEST,
};

pub const INVALID_ENDPOINT: ErrorInfo = ErrorInfo {
    message: INVALID_ENDPOINT_MSG,
    status_code: STATUS_UNPROCESSABLE_ENTITY, // 422 - valid format, semantic error
};

pub const INVALID_PARAMS: ErrorInfo = ErrorInfo {
    message: INVALID_PARAMS_MSG,
    status_code: STATUS_BAD_REQUEST, // 400 - malformed parameters
};

pub const INVALID_SOLUTION: ErrorInfo = ErrorInfo {
    message: INVALID_SOLUTION_MSG,
    status_code: STATUS_UNPROCESSABLE_ENTITY, // 422 - wrong solution
};

pub const CHALLENGE_EXPIRED: ErrorInfo = ErrorInfo {
    message: CHALLENGE_EXPIRED_MSG,
    status_code: STATUS_GONE, // 410 Gone - resource no longer available
};

pub const PUB_KEY_FAIL: ErrorInfo = ErrorInfo {
    message: PUB_KEY_FAIL_MSG,
    status_code: STATUS_INTERNAL_SERVER_ERROR, // 500 - server configuration issue
};

pub const SIG_KEY_FAIL: ErrorInfo = ErrorInfo {
    message: SIG_KEY_FAIL_MSG,
    status_code: STATUS_INTERNAL_SERVER_ERROR, // 500 - server configuration issue
};

pub const SIGNATURE_FAIL: ErrorInfo = ErrorInfo {
    message: SIGNATURE_FAIL_MSG,
    status_code: STATUS_UNPROCESSABLE_ENTITY, // 422 - invalid signature
};

// Backwards compatibility - keep the simple message constants for now
pub const MAX_TIME_DIFF_MS: i64 = 3 * 10000; // 3 * 10,000 milliseconds = 30 seconds

#[allow(dead_code)]
pub const CONFIG_ERROR: &str = "Invalid configuration";

#[allow(dead_code)]
pub const MAX_ITERATIONS: &str = "Maximum solving iterations reached without finding solution";

#[allow(dead_code)]
pub const NETWORK_ERROR: &str = "Network request failed";

#[allow(dead_code)]
pub const TIMEOUT_ERROR: &str = "Operation timed out";

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ErrorHandler {
    #[error("API error ({status}): {message}")]
    Api {
        /// HTTP status code returned by the API.
        status:  u16,
        /// Error message from the API response.
        message: String
    },
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
    #[error("Challenge processing error: {0}")]
    Challenge(String),
    #[error("Challenge solving failed: {0}")]
    ChallengeSolvingError(String),
    #[error("Challenge verification failed: {0}")]
    ChallengeVerificationError(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("Internal server error")]
    InternalError,
    #[error("Invalid request format: {0}")]
    InvalidRequest(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Network request failed: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("Resource not found: {0}")]
    NotFoundError(String),
    #[error("Permission denied: {0}")]
    PermissionError(String),
    #[error("Processing failed: {0}")]
    ProcessingError(String),
    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Operation timed out after {duration:?}")]
    TimeoutError { duration: Duration },
    #[cfg(feature = "toml")]
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),
}

/// Converts `ErrorHandler` into an `axum::response::Response`.
///
/// This implementation allows `ErrorHandler` to be used
/// as a response type in Axum handlers in ironshield-api.
impl IntoResponse for ErrorHandler {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ErrorHandler::InvalidRequest(message) => {
                (StatusCode::BAD_REQUEST, message)
            },
            ErrorHandler::ProcessingError(message) => {
                (StatusCode::UNPROCESSABLE_ENTITY, message)
            },
            ErrorHandler::SerializationError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Data processing error".to_string())
            },
            ErrorHandler::InternalError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            _ => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Unknown Error".to_string())
            }
        };

        let body: Json<serde_json::Value> = Json(serde_json::json!({
            "error":   error_message,
            "success": false,
        }));

        (status, body).into_response()
    }
}

#[allow(dead_code)]
impl ErrorHandler {
    /// # Arguments
    /// * `status`:  The HTTP status code from the API
    ///              response.
    /// * `message`: The error message that corresponds
    ///              to the API error.
    ///
    /// # Returns
    /// * `Self`: A new `CliError::Api` variant.
    pub fn api_error(
        status:  u16,
        message: impl Into<String>
    ) -> Self {
        Self::Api { status, message: message.into() }
    }

    /// # Arguments
    /// * `message`: The error message thrown on the event
    ///              authentication fails.
    ///
    /// # Returns
    /// * `Self`: An `ErrorHandler::AuthenticationError` passed with
    ///           the argument provided to this function.
    pub fn authentication_error(
        message: impl Into<String>
    ) -> Self {
        Self::AuthenticationError(message.into())
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

    /// # Arguments
    /// * `message`: The error message thrown on the event
    ///              solving a challenge fails.
    ///
    /// # Returns
    /// * `Self`: An `ErrorHandler::ChallengeSolvingError` passed with
    ///           an argument provided to this function.
    pub fn challenge_solving_error(
        message: impl Into<String>
    ) -> Self {
        Self::ChallengeSolvingError(message.into())
    }

    /// # Arguments
    /// * `message`: The error message thrown on the event
    ///              verification of a challenge fails.
    ///
    /// # Returns
    /// * `Self`: An `ErrorHandler::ChallengeVerificationError` passed
    ///           with the argument provided to this function.
    pub fn challenge_verification_error(
        message: impl Into<String>
    ) -> Self {
        Self::ChallengeVerificationError(message.into())
    }

    /// # Arguments
    /// * `message`: The error message thrown on the event
    ///              configuration fails.
    ///
    /// # Returns
    /// * `Self`: An `ErrorHandler::ConfigurationError` passed
    ///           with the argument provided to this function.
    pub fn config_error(
        message: impl Into<String>
    ) -> Self {
        Self::ConfigurationError(message.into())
    }

    /// # Arguments
    /// * `error`: A `reqwest` network error.
    ///
    /// # Returns
    /// * `Self`: An `ErrorHandler::NetworkError` passed with the
    ///           argument provided to this function.
    pub fn from_network_error(
        error: reqwest::Error
    ) -> Self {
        Self::NetworkError(error)
    }

    /// # Arguments
    /// * `message`: The error message thrown on the event
    ///              a `404` or "not found" error occurs.
    ///
    /// # Returns
    /// * `Self`: An `ErrorHandler::NotFoundError` passed with
    ///           the argument provided to this function.
    pub fn not_found_error(
        message: impl Into<String>
    ) -> Self {
        Self::NotFoundError(message.into())
    }

    /// # Arguments
    /// * `message`: The error message thrown on the event
    ///              that a permission error occurs.
    ///
    /// # Returns
    /// * `Self`: An `ErrorHandler::PermissionError` passed with
    ///           the argument provided to this function.
    pub fn permission_error(
        message: impl Into<String>
    ) -> Self {
        Self::PermissionError(message.into())
    }

    /// # Arguments
    /// * `message`: The error message thrown on the event
    ///              a rate limit error occurs.
    ///
    /// # Returns
    /// * `Self`: An `ErrorHandler::RateLimitError` passed with
    ///           the argument provided to this function.
    pub fn rate_limit_error(
        message: impl Into<String>
    ) -> Self {
        Self::RateLimitError(message.into())
    }

    /// # Arguments
    /// * `message`: The duration of the request.
    ///
    /// # Returns
    /// * `Self`: An `ErrorHandler::TimeoutError` passed with the
    ///           argument provided to this function.
    pub fn timeout(
        duration: Duration
    ) -> Self {
        Self::TimeoutError { duration }
    }
}