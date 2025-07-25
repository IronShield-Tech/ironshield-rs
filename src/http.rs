use reqwest::Client;

use crate::api::{ErrorHandler, ResultHandler};

use crate::constant::USER_AGENT;

use std::time::Duration;

/// Builder pattern for HTTP client configuration.
///
/// * `timeout`:              The request timeout duration.
/// * `user_agent`:           The user-agent header value.
/// * `accept_invalid_certs`: Whether to accept invalid SSL
///                           certs. Hopefully never `true`
///                           in a prod environment.
pub struct HttpClientBuilder {
    timeout:              Duration,
    user_agent:           String,
    accept_invalid_certs: bool,
}

impl Default for HttpClientBuilder {
    /// Default configuration for `HttpClientBuilder`.
    ///
    /// * Timeout: 30 seconds.
    /// * User-Agent: dependent on `constant::USER_AGENT`.
    /// * SSL certification validation: Enabled.
    fn default() -> Self {
        Self {
            timeout:              Duration::from_secs(30),
            user_agent:           USER_AGENT.to_string(),
            accept_invalid_certs: false,
        }
    }
}

#[allow(dead_code)]
impl HttpClientBuilder {
    /// # Returns
    /// `Self`: A new `HttpClientBuilder` with a default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// # Arguments
    /// * `duration`: The timeout duration for the HTTP request.
    ///
    /// # Returns
    /// * `Self`: The builder instance for method chaining.
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = duration;
        self
    }

    /// # Arguments
    /// * `agent`: The User-Agent string to use in a
    ///            request.
    ///
    /// # Returns
    /// * `Self`: The builder instance for method chaining.
    pub fn user_agent(mut self, agent: &str) -> Self {
        self.user_agent = agent.to_string();
        self
    }

    /// Please do not use this in prod.
    ///
    /// # Arguments
    /// * `accept`: Whether to accept invalid SSL certificates.
    ///
    /// # Returns
    /// * `Self`: The builder instance for method chaining.
    pub fn accept_invalid_certs(mut self, accept: bool) -> Self {
        self.accept_invalid_certs = accept;
        self
    }

    /// Builds the configured HTTP client.
    ///
    /// # Returns
    /// `ResultHandler<Client>`: A configured client or an
    ///                          error if the client could
    ///                          not be constructed.
    pub fn build(self) -> ResultHandler<Client> {
        Client::builder()
            .timeout(self.timeout)
            .user_agent(self.user_agent)
            .danger_accept_invalid_certs(self.accept_invalid_certs)
            .build()
            .map_err(ErrorHandler::from_network_error)
    }
} 