use notify::{RecommendedWatcher, RecursiveMode, Watcher};
#[cfg(test)]
use std::sync::Mutex;
use std::sync::RwLock;
use std::time::Duration;
use std::{fs, io, path::Path};

use crate::shared::error::CallerError;
use crate::{
    domain::api_config::ApiConfig, domain::caller_config::CallerConfig,
    domain::service_config::ServiceConfig,
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
#[cfg(test)]
static CONFIG_PATH_OVERRIDE: RwLock<Option<String>> = RwLock::new(None);
#[cfg(test)]
pub(crate) static TEST_STATE_LOCK: Mutex<()> = Mutex::new(());

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

    fn config_path() -> String {
        #[cfg(test)]
        {
            if let Ok(path) = CONFIG_PATH_OVERRIDE.read()
                && let Some(path) = path.as_ref()
            {
                return path.clone();
            }
        }

        CONFIG_PATH.to_string()
    }

    fn ensure_config_loaded() -> Result<(), CallerError> {
        let config_guard = CONFIG
            .read()
            .map_err(|_| CallerError::lock_poisoned("global config"))?;

        if config_guard.is_none() {
            drop(config_guard);
            return Self::reload_config();
        }

        Ok(())
    }

    pub fn get_config(
        service_name: &str,
        api_config: &str,
    ) -> Result<(ServiceConfig, ApiConfig), CallerError> {
        Self::ensure_config_loaded()?;

        let config_guard = CONFIG
            .read()
            .map_err(|_| CallerError::lock_poisoned("global config"))?;

        let config = config_guard
            .as_ref()
            .ok_or(CallerError::ConfigNotInitialized)?;

        let service_config = config
            .service_items
            .iter()
            .find(|s| s.api_name == service_name)
            .ok_or_else(|| CallerError::service_not_found(service_name))?;

        let api_config = service_config
            .api_items
            .iter()
            .find(|a| a.method == api_config)
            .ok_or_else(|| CallerError::api_not_found(service_name, api_config))?;

        Ok((service_config.clone(), api_config.clone()))
    }

    pub fn get_config_with_base_url(
        service_name: &str,
        api_config: &str,
    ) -> Result<(ServiceConfig, ApiConfig, String), CallerError> {
        Self::ensure_config_loaded()?;

        let config_guard = CONFIG
            .read()
            .map_err(|_| CallerError::lock_poisoned("global config"))?;

        let config = config_guard
            .as_ref()
            .ok_or(CallerError::ConfigNotInitialized)?;

        let service_config = config
            .service_items
            .iter()
            .find(|s| s.api_name == service_name)
            .ok_or_else(|| CallerError::service_not_found(service_name))?;

        let api_config = service_config
            .api_items
            .iter()
            .find(|a| a.method == api_config)
            .ok_or_else(|| CallerError::api_not_found(service_name, api_config))?;

        Ok((
            service_config.clone(),
            api_config.clone(),
            service_config.base_url.clone(),
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
            .map_err(|_| CallerError::lock_poisoned("global config"))?;

        config_guard
            .clone()
            .ok_or(CallerError::ConfigNotInitialized)
    }

    pub fn reload_config() -> Result<(), CallerError> {
        let config_path = Self::config_path();
        let config = Self::load_config_from_path(&config_path)?;
        Self::set_loaded_config(config)
    }

    pub fn load_config() -> Result<CallerConfig, CallerError> {
        let config_path = Self::config_path();
        Self::load_config_from_path(&config_path)
    }

    pub fn load_config_from_path(path: &str) -> Result<CallerConfig, CallerError> {
        let config_content = fs::read_to_string(path).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => CallerError::config_file_not_found(path),
            _ => CallerError::IoError(format!("Failed to read config file {}: {}", path, err)),
        })?;

        let format = ConfigFormat::detect_from_path(path)
            .ok_or_else(|| CallerError::unsupported_config_format(path))?;

        Self::parse_config(&config_content, format, path)
    }

    pub fn load_config_from_path_with_format(
        path: &str,
        format: ConfigFormat,
    ) -> Result<CallerConfig, CallerError> {
        let config_content = fs::read_to_string(path).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => CallerError::config_file_not_found(path),
            _ => CallerError::IoError(format!("Failed to read config file {}: {}", path, err)),
        })?;

        Self::parse_config(&config_content, format, path)
    }

    pub fn init_with_config(config: CallerConfig) {
        Self::set_loaded_config(config).unwrap();
    }

    pub fn start_watching(debounce_duration: Duration) -> Result<(), CallerError> {
        let config_path = Self::config_path();
        let path = Path::new(&config_path);

        if !path.exists() {
            return Err(CallerError::config_file_not_found(config_path));
        }

        let watch_path = path.parent().unwrap_or(path);
        let debounce = debounce_duration;

        let mut watcher = RecommendedWatcher::new(
            move |result: notify::Result<notify::Event>| {
                if result.is_ok() {
                    Self::handle_config_change(debounce);
                }
            },
            notify::Config::default(),
        )
        .map_err(|e| CallerError::config_watch_error(&config_path, e.to_string()))?;

        watcher
            .watch(watch_path, RecursiveMode::NonRecursive)
            .map_err(|e| CallerError::config_watch_error(&config_path, e.to_string()))?;

        {
            let mut watcher_guard = WATCHER.write().unwrap();
            *watcher_guard = Some(watcher);
        }

        Ok(())
    }

    pub fn stop_watching() {
        let mut watcher_guard = WATCHER.write().unwrap();
        *watcher_guard = None;
    }

    pub fn is_watching() -> bool {
        let watcher_guard = WATCHER.read().unwrap();
        watcher_guard.is_some()
    }

    fn handle_config_change(debounce_duration: Duration) {
        std::thread::sleep(debounce_duration);
        if let Err(e) = Self::reload_config() {
            eprintln!("Failed to reload config: {}", e);
        } else {
            println!("[Caller] Configuration reloaded successfully");
        }
    }

    fn set_loaded_config(config: CallerConfig) -> Result<(), CallerError> {
        let mut config_guard = CONFIG
            .write()
            .map_err(|_| CallerError::lock_poisoned("global config"))?;

        *config_guard = Some(config);
        Ok(())
    }

    pub fn try_get_config(
        service_name: &str,
        api_config: &str,
    ) -> Result<(ServiceConfig, ApiConfig), CallerError> {
        Self::get_config(service_name, api_config)
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
    /// use caller::ConfigLoader;
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
            .ok_or_else(|| CallerError::unsupported_config_format(output_path))?;

        // Serialize config to target format
        let content = match output_format {
            ConfigFormat::Json => serde_json::to_string_pretty(&config)
                .map_err(|e| CallerError::config_serialize_error("json", e.to_string()))?,
            ConfigFormat::Yaml => serde_yaml::to_string(&config)
                .map_err(|e| CallerError::config_serialize_error("yaml", e.to_string()))?,
            ConfigFormat::Toml => toml::to_string_pretty(&config)
                .map_err(|e| CallerError::config_serialize_error("toml", e.to_string()))?,
        };

        // Write to output file
        fs::write(output_path, content)
            .map_err(|e| CallerError::IoError(format!("Failed to write output file: {}", e)))?;

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
    /// use caller::{ConfigFileFormat as ConfigFormat, ConfigLoader};
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
            ConfigFormat::Json => serde_json::to_string_pretty(&config)
                .map_err(|e| CallerError::config_serialize_error("json", e.to_string()))?,
            ConfigFormat::Yaml => serde_yaml::to_string(&config)
                .map_err(|e| CallerError::config_serialize_error("yaml", e.to_string()))?,
            ConfigFormat::Toml => toml::to_string_pretty(&config)
                .map_err(|e| CallerError::config_serialize_error("toml", e.to_string()))?,
        };

        // Write to output file
        fs::write(output_path, content)
            .map_err(|e| CallerError::IoError(format!("Failed to write output file: {}", e)))?;

        Ok(())
    }

    fn parse_config(
        content: &str,
        format: ConfigFormat,
        path: &str,
    ) -> Result<CallerConfig, CallerError> {
        let config: CallerConfig = match format {
            ConfigFormat::Json => serde_json::from_str(content)
                .map_err(|e| CallerError::config_parse_error(path, "json", e.to_string())),
            ConfigFormat::Yaml => serde_yaml::from_str(content)
                .map_err(|e| CallerError::config_parse_error(path, "yaml", e.to_string())),
            ConfigFormat::Toml => toml::from_str(content)
                .map_err(|e| CallerError::config_parse_error(path, "toml", e.to_string())),
        }?;

        config.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
impl ConfigLoader {
    pub(crate) fn reset_state_for_test() {
        Self::stop_watching();

        if let Ok(mut config_guard) = CONFIG.write() {
            *config_guard = None;
        }

        if let Ok(mut path_guard) = CONFIG_PATH_OVERRIDE.write() {
            *path_guard = None;
        }
    }

    pub(crate) fn set_config_path_for_test(path: &Path) {
        let mut path_guard = CONFIG_PATH_OVERRIDE.write().unwrap();
        *path_guard = Some(path.display().to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_config;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_path(prefix: &str, extension: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "caller_{}_{}_{}.{}",
            prefix,
            std::process::id(),
            nanos,
            extension
        ))
    }

    fn write_config(path: &Path, base_url: &str) {
        let content = format!(
            r#"{{
  "authorizations": [],
  "service_items": [
    {{
      "api_name": "TestService",
      "base_url": "{base_url}",
      "api_items": [
        {{
          "method": "list",
          "url": "/items",
          "http_method": "GET",
          "param_type": "none"
        }}
      ]
    }}
  ]
}}"#
        );

        fs::write(path, content).unwrap();
    }

    #[test]
    fn test_init_config_loads_into_global_state() {
        let _guard = TEST_STATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let path = unique_test_path("init_config", "json");
        write_config(&path, "https://initial.example.com");

        ConfigLoader::reset_state_for_test();
        ConfigLoader::set_config_path_for_test(&path);

        assert!(!ConfigLoader::is_config_loaded());
        init_config().unwrap();
        assert!(ConfigLoader::is_config_loaded());

        let config = ConfigLoader::get_full_config().unwrap();
        assert_eq!(
            config.service_items[0].base_url,
            "https://initial.example.com"
        );

        ConfigLoader::reset_state_for_test();
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_handle_config_change_reloads_file_changes() {
        let _guard = TEST_STATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let path = unique_test_path("watch_config", "json");
        write_config(&path, "https://before.example.com");

        ConfigLoader::reset_state_for_test();
        ConfigLoader::set_config_path_for_test(&path);
        init_config().unwrap();
        write_config(&path, "https://after.example.com");
        ConfigLoader::handle_config_change(Duration::from_millis(0));

        let config = ConfigLoader::get_full_config().unwrap();
        assert_eq!(
            config.service_items[0].base_url,
            "https://after.example.com"
        );

        ConfigLoader::reset_state_for_test();
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_load_config_rejects_invalid_http_method_during_parse() {
        let _guard = TEST_STATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let path = unique_test_path("invalid_http_method", "json");
        let content = r#"{
  "authorizations": [],
  "service_items": [
    {
      "api_name": "TestService",
      "base_url": "https://example.com",
      "api_items": [
        {
          "method": "list",
          "url": "/items",
          "http_method": "FETCH",
          "param_type": "none"
        }
      ]
    }
  ]
}"#;

        fs::write(&path, content).unwrap();

        let err = ConfigLoader::load_config_from_path(path.to_string_lossy().as_ref())
            .expect_err("invalid http method should fail during config load");
        assert!(matches!(err, CallerError::ConfigParseError { .. }));

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_load_config_rejects_invalid_param_type_during_parse() {
        let _guard = TEST_STATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let path = unique_test_path("invalid_param_type", "json");
        let content = r#"{
  "authorizations": [],
  "service_items": [
    {
      "api_name": "TestService",
      "base_url": "https://example.com",
      "api_items": [
        {
          "method": "list",
          "url": "/items",
          "http_method": "GET",
          "param_type": "none,json"
        }
      ]
    }
  ]
}"#;

        fs::write(&path, content).unwrap();

        let err = ConfigLoader::load_config_from_path(path.to_string_lossy().as_ref())
            .expect_err("invalid param type should fail during config load");
        assert!(matches!(err, CallerError::ConfigParseError { .. }));

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_load_config_rejects_duplicate_param_type_during_parse() {
        let _guard = TEST_STATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let path = unique_test_path("duplicate_param_type", "json");
        let content = r#"{
  "authorizations": [],
  "service_items": [
    {
      "api_name": "TestService",
      "base_url": "https://example.com",
      "api_items": [
        {
          "method": "list",
          "url": "/items",
          "http_method": "GET",
          "param_type": "query,query"
        }
      ]
    }
  ]
}"#;

        fs::write(&path, content).unwrap();

        let err = ConfigLoader::load_config_from_path(path.to_string_lossy().as_ref())
            .expect_err("duplicate param type should fail during config load");
        assert!(matches!(err, CallerError::ConfigParseError { .. }));

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_load_config_rejects_invalid_endpoint_url_during_parse() {
        let _guard = TEST_STATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let path = unique_test_path("invalid_endpoint_url", "json");
        let content = r#"{
  "authorizations": [],
  "service_items": [
    {
      "api_name": "TestService",
      "base_url": "https://example.com",
      "api_items": [
        {
          "method": "list",
          "url": "items",
          "http_method": "GET",
          "param_type": "none"
        }
      ]
    }
  ]
}"#;

        fs::write(&path, content).unwrap();

        let err = ConfigLoader::load_config_from_path(path.to_string_lossy().as_ref())
            .expect_err("invalid endpoint url should fail during config load");
        assert!(matches!(err, CallerError::InvalidUrl { .. }));

        fs::remove_file(path).ok();
    }
}
