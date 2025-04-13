use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;

use crate::errors::{GoaError, GoaResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct GoaConfig {
    pub server: ServerConfig,
    pub directories: DirectoryConfig,
    pub performance: PerformanceConfig,
    pub ssg: SsgConfig,
    pub meta: MetaConfig,
    pub cdn: CdnConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: String,
    #[serde(rename = "devMode")]
    pub dev_mode: bool,
    #[serde(rename = "isBuiltSystem")]
    pub is_built_system: bool,
    #[serde(rename = "liveReload")]
    pub live_reload: bool,
    #[serde(rename = "enableCORS")]
    pub enable_cors: bool,
    #[serde(rename = "allowedOrigins")]
    pub allowed_origins: Vec<String>,
    #[serde(rename = "rateLimit")]
    pub rate_limit: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoryConfig {
    #[serde(rename = "appDir")]
    pub app_dir: String,
    #[serde(rename = "staticDir")]
    pub static_dir: String,
    #[serde(rename = "layoutPath")]
    pub layout_path: String,
    #[serde(rename = "componentDir")]
    pub component_dir: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceConfig {
    #[serde(rename = "templateCache")]
    pub template_cache: bool,
    #[serde(rename = "inMemoryJS")]
    pub in_memory_js: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SsgConfig {
    pub enabled: bool,
    #[serde(rename = "cacheEnabled")]
    pub cache_enabled: bool,
    pub directory: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetaConfig {
    #[serde(rename = "appName")]
    pub app_name: String,
    #[serde(rename = "defaultMetaTags")]
    pub default_meta_tags: MetaTags,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetaTags {
    pub viewport: String,
    pub description: String,
    #[serde(rename = "og:title")]
    pub og_title: String,
    #[serde(rename = "og:type")]
    pub og_type: String,
    #[serde(rename = "twitter:card")]
    pub twitter_card: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CdnConfig {
    #[serde(rename = "useCDN")]
    pub use_cdn: bool,
    pub tailwind: String,
    pub jquery: String,
    pub alpine: String,
    #[serde(rename = "petiteVue")]
    pub petite_vue: String,
}

impl GoaConfig {
    pub fn load(path: impl AsRef<Path>) -> GoaResult<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(GoaError::Configuration(
                format!("Configuration file not found at {}", path.display())
            ));
        }

        let config_str = fs::read_to_string(path)
            .map_err(|e| GoaError::Configuration(
                format!("Failed to read config file: {}", e)
            ))?;

        serde_json::from_str(&config_str)
            .map_err(|e| GoaError::Configuration(
                format!("Failed to parse config file: {}", e)
            ))
    }

    #[allow(dead_code)]
    pub fn save(&self, path: impl AsRef<Path>) -> GoaResult<()> {
        let path = path.as_ref();
        let parent = path.parent().ok_or_else(|| {
            GoaError::InvalidPath(format!("Invalid config path: {}", path.display()))
        })?;

        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| GoaError::Io(e))?;
        }

        let config_json = serde_json::to_string_pretty(self)
            .map_err(|e| GoaError::Json(e))?;

        let mut file = fs::File::create(path)
            .map_err(|e| GoaError::Io(e))?;

        file.write_all(config_json.as_bytes())
            .map_err(|e| GoaError::Io(e))?;

        Ok(())
    }

    pub fn get_app_dir(&self) -> PathBuf {
        PathBuf::from(&self.directories.app_dir)
    }

    pub fn get_api_dir(&self) -> PathBuf {
        let mut api_dir = self.get_app_dir();
        api_dir.push("api");
        api_dir
    }

    pub fn get_components_dir(&self) -> PathBuf {
        PathBuf::from(&self.directories.component_dir)
    }
} 