use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct HttpClientSettings {
    pub connect_timeout_secs: u64,
    pub timeout_secs: u64,
    pub max_connections: usize,
    pub pool_enabled: bool,
    pub user_agent: String,
}

impl Default for HttpClientSettings {
    fn default() -> Self {
        Self {
            connect_timeout_secs: 30,
            timeout_secs: 60,
            max_connections: 100,
            pool_enabled: true,
            user_agent: format!("telegrab/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}
