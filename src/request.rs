use reqwest::Client;

use crate::api::{ErrorHandler, ResultHandler};
use ironshield_types::{
    chrono,
    IronShieldChallenge,
    IronShieldRequest,
    IronShieldChallengeResponse,
    IronShieldToken,
};

use crate::config::ClientConfig;
use crate::http::HttpClientBuilder;
use crate::response::ApiResponse;

pub struct IronShieldClient {
    config:      ClientConfig,
    http_client: Client,
}

impl IronShieldClient {
    /// Creates a new IronShield client with the provided configuration.
    ///
    /// # Arguments
    /// * `config`: The client configuration.
    ///
    /// # Return
    /// * `ResultHandler<Self>`: The initialized client or an error.
    ///
    /// # Example
    /// ```
    /// let config = ClientConfig::from_file("ironshield.toml")?;
    /// let client = IronShieldClient::new(config)?;
    /// ```
    pub fn new(config: ClientConfig) -> ResultHandler<Self> {
        if !config.api_base_url.starts_with("https://") {
            return Err(ErrorHandler::config_error(
                crate::api::INVALID_ENDPOINT
            ));
        }

        let http_client = HttpClientBuilder::new()
            .timeout(config.timeout)
            .build()?;

        Ok(Self {
            config,
            http_client
        })
    }

    /// Fetches a challenge from the IronShield API.
    ///
    /// # Arguments
    /// * `endpoint`: The protected endpoint URL to access.
    ///
    /// # Returns
    /// * `ResultHandler<IronShieldChallenge>`: The challenge to solve.
    ///
    /// # Examples
    /// ```
    /// let challenge = client.fetch_challenge("https://example.com/protected").await?;
    /// println!("Challenge difficulty: {}", challenge.recommended_attempts);
    /// ```
    pub async fn fetch_challenge(
        &self,
        endpoint: &str
    ) -> ResultHandler<IronShieldChallenge> {
        let request = IronShieldRequest::new(
            endpoint.to_string(),
            chrono::Utc::now().timestamp_millis(),
        );

        let response = self.make_api_request("/request", &request).await?;
        let api_response = ApiResponse::from_json(response)?;

        api_response.extract_challenge()
    }

    pub async fn submit_solution(
        &self,
        solution: &IronShieldChallengeResponse,
    ) -> ResultHandler<IronShieldToken> {
        let response = self.make_api_request("/response", solution).await?;
        let api_response = ApiResponse::from_json(response)?;

        api_response.extract_token()
    }

    /// Makes a standardized API request to the IronShield API service.
    ///
    /// # Arguments
    /// * `path`: The API endpoint path (e.g., "/request" or "/response").
    /// * `body`: The request payload to send to the API.
    ///
    /// # Returns
    /// * `ResultHandler<serde_json::Value>`: The parsed JSON response
    ///                                       or an error if the
    ///                                       request fails.
    async fn make_api_request<T: serde::Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> ResultHandler<serde_json::Value> {
        let response = self
            .http_client
            .post(&format!("{}{}", self.config.api_base_url, path))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(ErrorHandler::from_network_error)?;

        if !response.status().is_success() {
            return Err(ErrorHandler::ProcessingError(format!(
                "API request failed with status: {}",
                response.status()
            )))
        }

        let json_response = response.json().await.map_err(ErrorHandler::from_network_error)?;

        Ok(json_response)
    }
} 