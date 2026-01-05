use crate::configuration::HttpClientSettings;
use crate::model::entity::doc::TelegraphPost;
use anyhow::Context;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct HttpClientManager {
    client: Arc<Client>,
    config: Arc<RwLock<HttpClientSettings>>,
}

impl HttpClientManager {
    pub fn new(config: Option<HttpClientSettings>) -> Self {
        let config = config.unwrap_or_default();
        tracing::debug!("Creating HTTP client with settings: {:?}", config);
        let client_builder = Client::builder()
            .connect_timeout(Duration::from_secs(config.connect_timeout_secs))
            .timeout(Duration::from_secs(config.timeout_secs))
            .user_agent(&config.user_agent);
        let client = if config.pool_enabled {
            client_builder
                .pool_max_idle_per_host(config.max_connections)
                .build()
                .expect("Failed to create HTTP client with connection pool")
        } else {
            client_builder
                .build()
                .expect("Failed to create HTTP client")
        };
        Self {
            client: Arc::new(client),
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub fn client(&self) -> Arc<Client> {
        self.client.clone()
    }
    pub async fn config(&self) -> HttpClientSettings {
        let config = self.config.read().await;
        config.clone()
    }
    pub async fn update_config(&self, new_config: HttpClientSettings) -> Result<(), String> {
        tracing::info!("Updating HTTP client config to: {:?}", new_config);
        let client_builder = Client::builder()
            .connect_timeout(Duration::from_secs(new_config.connect_timeout_secs))
            .timeout(Duration::from_secs(new_config.timeout_secs))
            .user_agent(&new_config.user_agent);
        // fixme: update client
        let _client = if new_config.pool_enabled {
            client_builder
                .pool_max_idle_per_host(new_config.max_connections)
                .build()
                .map_err(|e| format!("Failed to create HTTP client with connection pool: {}", e))?
        } else {
            client_builder
                .build()
                .map_err(|e| format!("Failed to create HTTP client: {}", e))?
        };
        {
            let mut config = self.config.write().await;
            *config = new_config;
            // todo: update client
        }

        tracing::warn!(
            "HTTP client config updated, new config will take effect after next restart"
        );
        Ok(())
    }
    pub async fn download_file<P:AsRef<Path>>(
        &self,
        url: &str,
        save_path: P,
    ) -> Result<DownloadResult, DownloadError> {
        let save_path_ref = save_path.as_ref();
        tracing::info!("Downloading file: {} -> {}", url, save_path_ref.display());
        let start_time = Instant::now();

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| DownloadError::NetworkError(format!("Request failed: {}", e)))?;
        let status = response.status();
        if !status.is_success() {
            return Err(DownloadError::HTTPError(
                status.as_u16(),
                status.to_string(),
            ));
        }
        let content_length = response.content_length().unwrap_or(0);
        tracing::info!("file size: {} bytes", content_length);
        let bytes = response
            .bytes()
            .await
            .map_err(|e| DownloadError::IOError(format!("Failed to read response bytes: {}", e)))?;
        tokio::fs::write(save_path_ref, &bytes)
            .await
            .map_err(|e| DownloadError::IOError(format!("Failed to write file: {}", e)))?;

        let duration = start_time.elapsed();
        let speed = if duration.as_secs() > 0 {
            content_length as f64 / duration.as_secs_f64()
        } else {
            content_length as f64
        };
        tracing::info!(
            "Downloaded {} bytes in {:?}, speed: {:.2} bytes/sec",
            bytes.len(),
            duration,
            speed
        );

        Ok(DownloadResult {
            url: url.to_string(),
            size: content_length,
            save_path: save_path_ref.to_string_lossy().to_string(),
            duration,
            speed: speed as u64,
        })
    }
    pub async fn parse_telegraph_post(&self, url: &str) -> crate::Result<TelegraphPost> {
        // 获取网页内容
        let html_content = self.client.get(url).send().await?.text().await?;

        // 解析HTML
        let document = Html::parse_document(&html_content);

        // 提取标题
        let title_selector = Selector::parse("h1").expect("Failed to parse h1 selector");
        let title = document
            .select(&title_selector)
            .next()
            .context("Failed to find title")
            .expect("Failed to find title element")
            .text()
            .collect::<String>()
            .trim()
            .to_string();

        // 提取日期
        let date_selector = Selector::parse("time").expect("Failed to parse time selector");
        let date = document.select(&date_selector).next().map(|element| {
            let s = element.text().collect::<String>().trim().to_string();
            let l = element.attr("datetime");
            if let Some(datetime) = l {
                datetime.to_string()
            } else {
                s
            }
        });

        // 提取图片URL
        let img_selector = Selector::parse("img").expect("Failed to parse img selector");
        let mut image_urls = Vec::new();
        let mut seen_urls = HashSet::new();

        for img_element in document.select(&img_selector) {
            if let Some(src) = img_element.value().attr("src") {
                let full_url = if src.starts_with("http") {
                    src.to_string()
                } else if src.starts_with("/") {
                    format!("https://telegra.ph{}", src)
                } else {
                    continue;
                };

                // 避免重复URL
                if seen_urls.insert(full_url.clone()) {
                    image_urls.push(full_url);
                }
            }
        }

        Ok(TelegraphPost {
            url: url.to_string(),
            title,
            date,
            image_urls,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub url: String,
    pub size: u64,
    pub save_path: String,
    pub duration: Duration,
    pub speed: u64,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum DownloadError {
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("HTTP error: {0}")]
    HTTPError(u16, String),
    #[error("IO error: {0}")]
    IOError(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}
