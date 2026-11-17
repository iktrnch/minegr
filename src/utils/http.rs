use anyhow::Context;
pub struct ApiClient;

impl ApiClient {
    /// Builds the shared HTTP client with Moxum identification and request timeouts.
    pub fn new() -> anyhow::Result<reqwest::blocking::Client> {
        reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION"),
                " (https://github.com/iktrnch/moxum)"
            ))
            .build()
            .context("Failed to build the API HTTP client")
    }
}
