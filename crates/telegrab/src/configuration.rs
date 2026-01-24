use crate::telemetry;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx_postgres::{PgConnectOptions, PgSslMode};

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    let environment_filename = format!("{}.yaml", environment.as_str());
    let settings = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(&environment_filename),
        ))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;
    settings.try_deserialize::<Settings>()
}
pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_string().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment.Use either `local` or `production`",
                other
            )),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub http_client: HttpClientSettings,
    pub worker: WorkerSettings,
    pub logger: LoggerSettings,
    pub redis_uri: SecretString,
    pub pic_dir: String,
    pub cbz_dir: String,
}

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
            user_agent: "telegraph/0.1.0".into(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct WorkerSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub count: usize,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub max_completed_tasks: usize,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub auto_cleanup_interval_secs: u64,
}

impl Default for WorkerSettings {
    fn default() -> Self {
        Self {
            count: 4,
            max_completed_tasks: 100,
            auto_cleanup_interval_secs: 60,
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ListenerType {
    Tcp,
    Uds,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ListenerConfig {
    #[serde(rename = "type")]
    pub listener_type: ListenerType,
    pub address: String,
}

impl Default for ListenerConfig {
    fn default() -> Self {
        Self {
            listener_type: ListenerType::Tcp,
            address: "127.0.0.1:9000".into(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApplicationSettings {
    pub listeners: Vec<ListenerConfig>,
    pub base_url: String,
}

impl Default for ApplicationSettings {
    fn default() -> Self {
        Self {
            listeners: vec![ListenerConfig::default()],
            base_url: "http://localhost:9000".into(),
        }
    }
}
#[derive(Deserialize, Debug, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.database_name)
    }

    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        if self.host.starts_with("/") {
            // Unix socket, so we don't need other options.
            return PgConnectOptions::new().host(&self.host);
        }
        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.username)
            .password(self.password.expose_secret())
            .ssl_mode(ssl_mode)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct LoggerSettings {
    pub pretty_backtrace: bool,
    pub level: telemetry::LogLevel,
    pub format: telemetry::Format,
}
