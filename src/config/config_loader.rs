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

static CONFIG_PATH: &str = "./caller.json";

static CONFIG: RwLock<Option<CallerConfig>> = RwLock::new(None);
static WATCHER: RwLock<Option<RecommendedWatcher>> = RwLock::new(None);
static WATCHER_TX: RwLock<Option<broadcast::Sender<()>>> = RwLock::new(None);

pub struct ConfigLoader;

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

    fn load_config_from_path(path: &str) -> Result<CallerConfig, Box<dyn Error>> {
        let config_content = fs::read_to_string(path)?;
        let caller_config = serde_json::from_str(&config_content)?;
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
                if let Ok(event) = result {
                    if event.kind.is_modify() || event.kind.is_create() {
                        let _ = tx_clone.send(());
                    }
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
}
