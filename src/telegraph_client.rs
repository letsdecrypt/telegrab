use reqwest::{Client, Proxy};
pub struct TelegraphClient {
    http_client: Client,
}

impl TelegraphClient {
    pub fn new(
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = Client::builder()
            .timeout(timeout)
            .build()
            .unwrap();
        Self {
            http_client,
        }
    }
    
    pub fn client(&self) -> &Client {
        &self.http_client
    }
}
