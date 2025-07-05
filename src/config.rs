use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub auth: AuthConfig,
    pub cache: CacheConfig,
    pub display: DisplayConfig,
    pub calendars: Vec<CalendarConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub credentials_path: String,
    pub token_cache_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub ttl_seconds: u64,
    pub max_entries: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub max_events: usize,
    pub date_format: String,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarConfig {
    pub id: String,
    pub name: String,
    pub color: String,
    pub enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auth: AuthConfig {
                credentials_path: "~/.config/callux/credentials.json".to_string(),
                token_cache_path: "~/.config/callux/token.json".to_string(),
            },
            cache: CacheConfig {
                ttl_seconds: 300,
                max_entries: 1000,
            },
            display: DisplayConfig {
                max_events: 10,
                date_format: "%Y-%m-%d %H:%M".to_string(),
                timezone: "local".to_string(),
            },
            calendars: vec![CalendarConfig {
                id: "primary".to_string(),
                name: "Personal".to_string(),
                color: "#1976d2".to_string(),
                enabled: true,
            }],
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let config_str = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let config_str = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, config_str)?;
        Ok(())
    }

    fn get_config_path() -> Result<PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        Ok(config_dir.join("callux").join("config.toml"))
    }

    pub fn expand_path(&self, path: &str) -> String {
        if path.starts_with("~/") {
            if let Some(home_dir) = dirs::home_dir() {
                return home_dir.join(&path[2..]).to_string_lossy().to_string();
            }
        }
        path.to_string()
    }
}
