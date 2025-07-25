pub mod config;
pub mod constant;
pub mod error;
pub mod http;
pub mod request;
pub mod response;
pub mod solve;
pub mod validate;

// Re-export key types for convenience
pub use config::ClientConfig;
pub use error::CliError;
pub use request::IronShieldClient;
pub use solve::{solve_challenge, SolveConfig, ProgressTracker};
pub use validate::validate_challenge;

// Re-export types from ironshield-types for convenience
pub use ironshield_types::{
    IronShieldChallenge,
    IronShieldChallengeResponse,
    IronShieldToken,
    IronShieldRequest,
}; 