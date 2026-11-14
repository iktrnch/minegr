use anyhow::Context;
pub struct ApiClient;

impl ApiClient {
    /// Builds the shared HTTP client with Minegr identification and request timeouts.
    pub fn new() -> anyhow::Result<reqwest::blocking::Client> {
        reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION"),
                " (https://github.com/iktrnch/minegr)"
            ))
            .build()
            .context("Failed to build the API HTTP client")
    }
}
