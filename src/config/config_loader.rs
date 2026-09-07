use notify::{RecommendedWatcher, RecursiveMode, Watcher};
#[cfg(test)]
use std::sync::Mutex;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use std::{fs, io, path::Path};

use crate::config::ConfigFormat;
use crate::shared::error::CallerError;
use crate::{
    domain::api_config::ApiConfig, domain::caller_config::CallerConfig,
    domain::service_config::ServiceConfig,
};

static CONFIG_PATH: &str = "./caller.json";

static CONFIG: RwLock<Option<CallerConfig>> = RwLock::new(None);
static WATCHER: RwLock<Option<RecommendedWatcher>> = RwLock::new(None);
static LAST_WATCH_ERROR: RwLock<Option<String>> = RwLock::new(None);
static WATCH_GENERATION: AtomicU64 = AtomicU64::new(0);
#[cfg(test)]
static CONFIG_PATH_OVERRIDE: RwLock<Option<String>> = RwLock::new(None);
#[cfg(test)]
pub(crate) static TEST_STATE_LOCK: Mutex<()> = Mutex::new(());

/// Synchronous loader and process-global configuration manager.
///
/// Prefer [`crate::Caller::from_path`] when configuration and authentication
/// must be isolated per client instance. The stateful methods on this type
/// support the crate-root convenience API.
pub struct ConfigLoader;

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigLoader {
    /// Construct the stateless loader value.
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

    /// Resolve and clone a service and endpoint from process-global configuration.
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

    /// Resolve global service/endpoint configuration together with its base URL.
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

    /// Return whether process-global configuration is currently initialized.
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

    /// Reload the default `./caller.json` into process-global configuration.
    pub fn reload_config() -> Result<(), CallerError> {
        let config_path = Self::config_path();
        let config = Self::load_config_from_path(&config_path)?;
        Self::set_loaded_config(config)
    }

    /// Load and validate the default `./caller.json` without changing global state.
    pub fn load_config() -> Result<CallerConfig, CallerError> {
        let config_path = Self::config_path();
        Self::load_config_from_path(&config_path)
    }

    /// Load and validate configuration, detecting format from the path extension.
    pub fn load_config_from_path(path: &str) -> Result<CallerConfig, CallerError> {
        let config_content = fs::read_to_string(path).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => CallerError::config_file_not_found(path),
            _ => CallerError::io(format!("reading config file '{path}'"), err),
        })?;

        let format = ConfigFormat::detect_from_path(path)
            .ok_or_else(|| CallerError::unsupported_config_format(path))?;

        Self::parse_config(&config_content, format, path)
    }

    /// Load and validate configuration using an explicit format.
    pub fn load_config_from_path_with_format(
        path: &str,
        format: ConfigFormat,
    ) -> Result<CallerConfig, CallerError> {
        let config_content = fs::read_to_string(path).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => CallerError::config_file_not_found(path),
            _ => CallerError::io(format!("reading config file '{path}'"), err),
        })?;

        Self::parse_config(&config_content, format, path)
    }

    /// Validate and install in-memory process-global configuration.
    pub fn init_with_config(config: CallerConfig) -> Result<(), CallerError> {
        config.validate()?;
        let mut config_guard = CONFIG
            .write()
            .map_err(|_| CallerError::lock_poisoned("global config"))?;
        *config_guard = Some(config);
        Ok(())
    }

    /// Watch the default config path and reload global state after debounced changes.
    ///
    /// Errors that happen in later filesystem callbacks can be read through
    /// [`Self::last_watch_error`]. Calling this again replaces the active watcher.
    pub fn start_watching(debounce_duration: Duration) -> Result<(), CallerError> {
        let config_path = Self::config_path();
        let path = Path::new(&config_path);

        if !path.exists() {
            return Err(CallerError::config_file_not_found(config_path));
        }

        let watch_path = path.parent().unwrap_or(path);
        let debounce = debounce_duration;

        let watched_file = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let mut watcher = RecommendedWatcher::new(
            move |result: notify::Result<notify::Event>| match result {
                Ok(event)
                    if event.paths.iter().any(|path| {
                        fs::canonicalize(path)
                            .map(|path| path == watched_file)
                            .unwrap_or_else(|_| path.ends_with(&watched_file))
                    }) =>
                {
                    Self::schedule_config_change(debounce);
                }
                Ok(_) => {}
                Err(error) => Self::set_last_watch_error(error.to_string()),
            },
            notify::Config::default(),
        )
        .map_err(|e| CallerError::config_watch_error(&config_path, e.to_string()))?;

        watcher
            .watch(watch_path, RecursiveMode::NonRecursive)
            .map_err(|e| CallerError::config_watch_error(&config_path, e.to_string()))?;

        {
            let mut watcher_guard = WATCHER
                .write()
                .map_err(|_| CallerError::lock_poisoned("config watcher"))?;
            *watcher_guard = Some(watcher);
        }
        Self::clear_last_watch_error();

        Ok(())
    }

    /// Stop the process-global configuration watcher, if one is active.
    pub fn stop_watching() {
        WATCH_GENERATION.fetch_add(1, Ordering::SeqCst);
        let mut watcher_guard = WATCHER
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *watcher_guard = None;
    }

    /// Return whether a process-global configuration watcher is active.
    pub fn is_watching() -> bool {
        let watcher_guard = WATCHER
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        watcher_guard.is_some()
    }

    /// Return the most recent asynchronous config-watch error.
    pub fn last_watch_error() -> Option<String> {
        LAST_WATCH_ERROR
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn schedule_config_change(debounce_duration: Duration) {
        let generation = WATCH_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
        std::thread::spawn(move || {
            std::thread::sleep(debounce_duration);
            if WATCH_GENERATION.load(Ordering::SeqCst) != generation {
                return;
            }
            Self::reload_after_watch_event();
        });
    }

    #[cfg(test)]
    fn handle_config_change(debounce_duration: Duration) {
        std::thread::sleep(debounce_duration);
        Self::reload_after_watch_event();
    }

    fn reload_after_watch_event() {
        if let Err(error) = Self::reload_config() {
            Self::set_last_watch_error(error.to_string());
        } else {
            Self::clear_last_watch_error();
        }
    }

    fn set_last_watch_error(error: String) {
        let mut last_error = LAST_WATCH_ERROR
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *last_error = Some(error);
    }

    fn clear_last_watch_error() {
        let mut last_error = LAST_WATCH_ERROR
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *last_error = None;
    }

    fn set_loaded_config(config: CallerConfig) -> Result<(), CallerError> {
        let mut config_guard = CONFIG
            .write()
            .map_err(|_| CallerError::lock_poisoned("global config"))?;

        *config_guard = Some(config);
        Ok(())
    }

    /// Compatibility alias for [`Self::get_config`].
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
            ConfigFormat::Yaml => serde_yaml_ng::to_string(&config)
                .map_err(|e| CallerError::config_serialize_error("yaml", e.to_string()))?,
            ConfigFormat::Toml => toml::to_string_pretty(&config)
                .map_err(|e| CallerError::config_serialize_error("toml", e.to_string()))?,
        };

        // Write to output file
        fs::write(output_path, content).map_err(|error| {
            CallerError::io(format!("writing config file '{output_path}'"), error)
        })?;

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
    /// use caller::{ConfigFormat, ConfigLoader};
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
            ConfigFormat::Yaml => serde_yaml_ng::to_string(&config)
                .map_err(|e| CallerError::config_serialize_error("yaml", e.to_string()))?,
            ConfigFormat::Toml => toml::to_string_pretty(&config)
                .map_err(|e| CallerError::config_serialize_error("toml", e.to_string()))?,
        };

        // Write to output file
        fs::write(output_path, content).map_err(|error| {
            CallerError::io(format!("writing config file '{output_path}'"), error)
        })?;

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
            ConfigFormat::Yaml => serde_yaml_ng::from_str(content)
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

        Self::clear_last_watch_error();
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
    fn config_defaults_authorizations_and_rejects_unknown_fields() {
        let valid = r#"{
          "service_items": [{
            "api_name": "TestService",
            "base_url": "https://example.com",
            "api_items": []
          }]
        }"#;
        let config = ConfigLoader::parse_config(valid, ConfigFormat::Json, "inline.json").unwrap();
        assert!(config.authorizations.is_empty());

        let typo = r#"{
          "service_items": [],
          "service_itmes": []
        }"#;
        let error = ConfigLoader::parse_config(typo, ConfigFormat::Json, "inline.json")
            .expect_err("configuration typos must not be ignored");
        assert!(matches!(error, CallerError::ConfigParseError { .. }));
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
