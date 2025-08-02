pub mod constant;
pub mod handler {
    pub mod error;
    pub mod result;
}

pub mod client {
    pub mod config;
    pub mod http;
    pub mod request;
    pub mod response;
    pub mod solve;
    pub mod validate;
}

pub use constant::USER_AGENT;
pub use client::config::ClientConfig;
pub use client::request::IronShieldClient;
pub use client::solve::{
    solve_challenge,
    SolveConfig,
    ProgressTracker
};
pub use client::validate::validate_challenge;

pub use ironshield_types::{
    IronShieldChallenge,
    IronShieldChallengeResponse,
    IronShieldToken,
    IronShieldRequest,
};