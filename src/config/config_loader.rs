use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::RwLock;
use std::time::Duration;
use std::{error::Error, fs, path::Path};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;

use crate::shared::error::CallerError;
use crate::{
    domain::api_item::ApiItem, domain::caller_config::CallerConfig,
    domain::service_item::ServiceItem,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Json,
    Yaml,
    Toml,
}

impl ConfigFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "json" => Some(ConfigFormat::Json),
            "yaml" | "yml" => Some(ConfigFormat::Yaml),
            "toml" => Some(ConfigFormat::Toml),
            _ => None,
        }
    }

    pub fn detect_from_path(path: &str) -> Option<Self> {
        Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }
}

static CONFIG_PATH: &str = "./caller.json";

static CONFIG: RwLock<Option<CallerConfig>> = RwLock::new(None);
static WATCHER: RwLock<Option<RecommendedWatcher>> = RwLock::new(None);
static WATCHER_TX: RwLock<Option<broadcast::Sender<()>>> = RwLock::new(None);

pub struct ConfigLoader;

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigLoader {
    pub fn new() -> Self {
        ConfigLoader
    }

    fn ensure_config_loaded() -> Result<(), CallerError> {
        let config_guard = CONFIG
            .read()
            .map_err(|_| CallerError::ConfigError("Config lock poisoned".to_string()))?;

        if config_guard.is_none() {
            drop(config_guard);
            return Self::reload_config();
        }

        Ok(())
    }

    pub fn get_config(
        service_name: &str,
        api_item: &str,
    ) -> Result<(ServiceItem, ApiItem), CallerError> {
        Self::ensure_config_loaded()?;

        let config_guard = CONFIG
            .read()
            .map_err(|_| CallerError::ConfigError("Config lock poisoned".to_string()))?;

        let config = config_guard
            .as_ref()
            .ok_or(CallerError::ConfigNotInitialized)?;

        let service_item = config
            .service_items
            .iter()
            .find(|s| s.api_name == service_name)
            .ok_or_else(|| CallerError::service_not_found(service_name))?;

        let api_item = service_item
            .api_items
            .iter()
            .find(|a| a.method == api_item)
            .ok_or_else(|| CallerError::api_not_found(service_name, api_item))?;

        Ok((service_item.clone(), api_item.clone()))
    }

    pub fn get_config_with_base_url(
        service_name: &str,
        api_item: &str,
    ) -> Result<(ServiceItem, ApiItem, String), CallerError> {
        Self::ensure_config_loaded()?;

        let config_guard = CONFIG
            .read()
            .map_err(|_| CallerError::ConfigError("Config lock poisoned".to_string()))?;

        let config = config_guard
            .as_ref()
            .ok_or(CallerError::ConfigNotInitialized)?;

        let service_item = config
            .service_items
            .iter()
            .find(|s| s.api_name == service_name)
            .ok_or_else(|| CallerError::service_not_found(service_name))?;

        let api_item = service_item
            .api_items
            .iter()
            .find(|a| a.method == api_item)
            .ok_or_else(|| CallerError::api_not_found(service_name, api_item))?;

        Ok((
            service_item.clone(),
            api_item.clone(),
            service_item.base_url.clone(),
        ))
    }

    pub fn is_config_loaded() -> bool {
        CONFIG
            .read()
            .map(|config| config.is_some())
            .unwrap_or(false)
    }

    /// Get the full configuration object
    pub fn get_full_config() -> Result<CallerConfig, CallerError> {
        Self::ensure_config_loaded()?;

        let config_guard = CONFIG
            .read()
            .map_err(|_| CallerError::ConfigError("Config lock poisoned".to_string()))?;

        config_guard
            .clone()
            .ok_or(CallerError::ConfigNotInitialized)
    }

    pub fn reload_config() -> Result<(), CallerError> {
        let config = Self::load_config_from_path(CONFIG_PATH)?;
        let mut config_guard = CONFIG
            .write()
            .map_err(|_| CallerError::ConfigError("Config lock poisoned".to_string()))?;

        *config_guard = Some(config);
        Ok(())
    }

    pub fn load_config() -> Result<CallerConfig, Box<dyn Error>> {
        Self::load_config_from_path(CONFIG_PATH)
    }

    pub fn load_config_from_path(path: &str) -> Result<CallerConfig, Box<dyn Error>> {
        let config_content = fs::read_to_string(path)?;
        
        let format = ConfigFormat::detect_from_path(path)
            .ok_or_else(|| CallerError::ConfigError(format!(
                "Unsupported config file format for path: {}. Supported formats: .json, .yaml, .yml, .toml",
                path
            )))?;
        
        let caller_config = match format {
            ConfigFormat::Json => {
                serde_json::from_str(&config_content)
                    .map_err(|e| CallerError::ConfigError(format!("Failed to parse JSON: {}", e)))?
            }
            ConfigFormat::Yaml => {
                serde_yaml::from_str(&config_content)
                    .map_err(|e| CallerError::ConfigError(format!("Failed to parse YAML: {}", e)))?
            }
            ConfigFormat::Toml => {
                toml::from_str(&config_content)
                    .map_err(|e| CallerError::ConfigError(format!("Failed to parse TOML: {}", e)))?
            }
        };
        
        Ok(caller_config)
    }

    pub fn load_config_from_path_with_format(
        path: &str,
        format: ConfigFormat,
    ) -> Result<CallerConfig, Box<dyn Error>> {
        let config_content = fs::read_to_string(path)?;
        
        let caller_config = match format {
            ConfigFormat::Json => {
                serde_json::from_str(&config_content)
                    .map_err(|e| CallerError::ConfigError(format!("Failed to parse JSON: {}", e)))?
            }
            ConfigFormat::Yaml => {
                serde_yaml::from_str(&config_content)
                    .map_err(|e| CallerError::ConfigError(format!("Failed to parse YAML: {}", e)))?
            }
            ConfigFormat::Toml => {
                toml::from_str(&config_content)
                    .map_err(|e| CallerError::ConfigError(format!("Failed to parse TOML: {}", e)))?
            }
        };
        
        Ok(caller_config)
    }

    pub fn init_with_config(config: CallerConfig) {
        let mut config_guard = CONFIG.write().unwrap();
        *config_guard = Some(config);
    }

    pub fn start_watching(debounce_duration: Duration) -> Result<(), CallerError> {
        let path = Path::new(CONFIG_PATH);

        if !path.exists() {
            return Err(CallerError::ConfigError(format!(
                "Config file not found: {}",
                CONFIG_PATH
            )));
        }

        let tx_clone = {
            let (tx, _) = broadcast::channel(1);
            tx.clone()
        };

        let mut watcher = RecommendedWatcher::new(
            move |result: notify::Result<notify::Event>| {
                if let Ok(event) = result
                    && (event.kind.is_modify() || event.kind.is_create())
                {
                    let _ = tx_clone.send(());
                }
            },
            notify::Config::default(),
        )?;

        watcher
            .watch(path, RecursiveMode::NonRecursive)
            .map_err(|e| CallerError::ConfigError(format!("Failed to watch config file: {}", e)))?;

        let (tx, rx) = broadcast::channel(1);

        {
            let mut watcher_guard = WATCHER.write().unwrap();
            let mut tx_guard = WATCHER_TX.write().unwrap();
            *watcher_guard = Some(watcher);
            *tx_guard = Some(tx);
        }

        let debounce_ms = debounce_duration.as_millis() as u64;
        tokio::spawn(async move {
            Self::watch_for_changes(rx, debounce_ms).await;
        });

        Ok(())
    }

    async fn watch_for_changes(mut rx: broadcast::Receiver<()>, debounce_ms: u64) {
        loop {
            match rx.recv().await {
                Ok(()) => {
                    tokio::time::sleep(Duration::from_millis(debounce_ms)).await;
                    if let Err(e) = Self::reload_config() {
                        eprintln!("Failed to reload config: {}", e);
                    } else {
                        println!("[Caller] Configuration reloaded successfully");
                    }
                }
                Err(RecvError::Lagged(_)) => {
                    Self::reload_config().ok();
                }
                Err(_) => break,
            }
        }
    }

    pub fn stop_watching() {
        let mut watcher_guard = WATCHER.write().unwrap();
        let mut tx_guard = WATCHER_TX.write().unwrap();
        *watcher_guard = None;
        *tx_guard = None;
    }

    pub fn is_watching() -> bool {
        let watcher_guard = WATCHER.read().unwrap();
        watcher_guard.is_some()
    }

    pub fn try_get_config(
        service_name: &str,
        api_item: &str,
    ) -> Result<(ServiceItem, ApiItem), CallerError> {
        Self::get_config(service_name, api_item)
    }

    /// Convert config file from one format to another
    /// 
    /// # Arguments
    /// * `input_path` - Path to the input config file
    /// * `output_path` - Path to write the converted config file
    /// 
    /// # Example
    /// 
    /// ```rust,ignore
    /// use caller::config::config_loader::ConfigLoader;
    /// 
    /// // Convert JSON to YAML
    /// let result = ConfigLoader::convert_config("config.json", "config.yaml");
    /// 
    /// // Convert YAML to TOML
    /// let result = ConfigLoader::convert_config("config.yaml", "config.toml");
    /// ```
    pub fn convert_config(input_path: &str, output_path: &str) -> Result<(), CallerError> {
        // Load config from input path
        let config = Self::load_config_from_path(input_path)?;
        
        // Detect output format
        let output_format = ConfigFormat::detect_from_path(output_path)
            .ok_or_else(|| CallerError::ConfigError(format!(
                "Unsupported output config file format for path: {}. Supported formats: .json, .yaml, .yml, .toml",
                output_path
            )))?;
        
        // Serialize config to target format
        let content = match output_format {
            ConfigFormat::Json => {
                serde_json::to_string_pretty(&config)
                    .map_err(|e| CallerError::ConfigError(format!("Failed to serialize to JSON: {}", e)))?
            }
            ConfigFormat::Yaml => {
                serde_yaml::to_string(&config)
                    .map_err(|e| CallerError::ConfigError(format!("Failed to serialize to YAML: {}", e)))?
            }
            ConfigFormat::Toml => {
                toml::to_string_pretty(&config)
                    .map_err(|e| CallerError::ConfigError(format!("Failed to serialize to TOML: {}", e)))?
            }
        };
        
        // Write to output file
        fs::write(output_path, content)
            .map_err(|e| CallerError::ConfigError(format!("Failed to write output file: {}", e)))?;
        
        Ok(())
    }

    /// Convert config file from one format to another with explicit format specification
    /// 
    /// # Arguments
    /// * `input_path` - Path to the input config file
    /// * `output_path` - Path to write the converted config file
    /// * `output_format` - The target format for conversion
    /// 
    /// # Example
    /// 
    /// ```rust,ignore
    /// use caller::config::config_loader::{ConfigLoader, ConfigFormat};
    /// 
    /// // Convert to YAML regardless of output file extension
    /// let result = ConfigLoader::convert_config_with_format("config.json", "config.txt", ConfigFormat::Yaml);
    /// ```
    pub fn convert_config_with_format(
        input_path: &str,
        output_path: &str,
        output_format: ConfigFormat,
    ) -> Result<(), CallerError> {
        // Load config from input path
        let config = Self::load_config_from_path(input_path)?;
        
        // Serialize config to target format
        let content = match output_format {
            ConfigFormat::Json => {
                serde_json::to_string_pretty(&config)
                    .map_err(|e| CallerError::ConfigError(format!("Failed to serialize to JSON: {}", e)))?
            }
            ConfigFormat::Yaml => {
                serde_yaml::to_string(&config)
                    .map_err(|e| CallerError::ConfigError(format!("Failed to serialize to YAML: {}", e)))?
            }
            ConfigFormat::Toml => {
                toml::to_string_pretty(&config)
                    .map_err(|e| CallerError::ConfigError(format!("Failed to serialize to TOML: {}", e)))?
            }
        };
        
        // Write to output file
        fs::write(output_path, content)
            .map_err(|e| CallerError::ConfigError(format!("Failed to write output file: {}", e)))?;
        
        Ok(())
    }
}
