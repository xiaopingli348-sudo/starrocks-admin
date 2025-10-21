use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub cors: CorsConfig,
    pub logging: LoggingConfig,
    pub static_config: StaticConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expires_in: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CorsConfig {
    pub allow_origin: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct StaticConfig {
    pub enabled: bool,
    pub web_root: String,
}

impl Config {
    pub fn load() -> Result<Self, anyhow::Error> {
        // Try to load from config.toml
        if let Some(config_path) = Self::find_config_file() {
            Self::from_toml(&config_path)
        } else {
            Err(anyhow::anyhow!("Configuration file not found. Please create conf/config.toml"))
        }
    }

    fn find_config_file() -> Option<String> {
        let possible_paths = [
            "conf/config.toml",
            "config.toml",
            "./conf/config.toml",
            "./config.toml",
        ];

        for path in &possible_paths {
            if Path::new(path).exists() {
                return Some(path.to_string());
            }
        }
        None
    }

    fn from_toml(path: &str) -> Result<Self, anyhow::Error> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}


impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite://tmp/starrocks-admin.db".to_string(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "dev-secret-key-change-in-production".to_string(),
            jwt_expires_in: "24h".to_string(),
        }
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allow_origin: "http://localhost:4200".to_string(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info,starrocks_admin_backend=debug".to_string(),
            file: Some("logs/starrocks-admin.log".to_string()),
        }
    }
}

impl Default for StaticConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            web_root: "web".to_string(),
        }
    }
}

