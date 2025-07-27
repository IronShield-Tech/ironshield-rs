use crate::{
    solve_challenge, 
    ClientConfig, 
    IronShieldClient,
    result::ResultHandler
};

use ironshield_types::IronShieldToken;

/// Fetches a challenge, solves it, and submits the solution for validation.
///
/// # Arguments
/// * `client`:          An instance of `IronShieldClient` to communicate with the API.
/// * `config`:          The client configuration.
/// * `endpoint`:        The protected endpoint URL to get a challenge for.
/// * `use_multithread`: A boolean indicating whether to use multithreaded solving.
///
/// # Returns
/// * `ResultHandler<IronShieldToken>`: An `IronShieldToken` if successful,
///                                     or an error.
pub async fn validate_challenge(
    client:          &IronShieldClient,
    config:          &ClientConfig,
    endpoint:        &str,
    use_multithread: bool,
) -> ResultHandler<IronShieldToken> {
    let challenge = client.fetch_challenge(endpoint).await?;
    let  solution = solve_challenge(challenge, config, use_multithread, None).await?;
    let     token = client.submit_solution(&solution).await?;

    Ok(token)
} 