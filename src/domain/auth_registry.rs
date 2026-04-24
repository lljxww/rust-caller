use std::collections::HashMap;
use std::sync::RwLock;

use super::auth_trait::{AuthContext, AuthProvider, Authenticator};
use crate::shared::error::CallerError;

/// Global authentication registry for managing authenticators
///
/// This registry stores and manages all authentication providers by name,
/// supporting both trait-based and closure-based authentication.
///
/// # Thread Safety
/// The registry uses RwLock for thread-safe concurrent access.
///
/// # Caching
/// Providers are cached in Arc for efficient reuse across requests.
///
/// # Example
/// ```rust
/// use caller::{AuthRegistry, BearerAuth};
///
/// // Register a custom authenticator
/// AuthRegistry::register("my_auth", BearerAuth::new("token".to_string())).unwrap();
///
/// // Check if registered
/// assert!(AuthRegistry::contains("my_auth"));
/// # AuthRegistry::clear().unwrap();
/// ```
pub struct AuthRegistry {
    providers: RwLock<HashMap<String, AuthProvider>>,
}

impl AuthRegistry {
    /// Create a new empty registry
    fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
        }
    }

    /// Get the global registry instance (lazy initialized)
    fn global() -> &'static Self {
        static INSTANCE: std::sync::OnceLock<AuthRegistry> = std::sync::OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    /// Register an authenticator with a given name
    ///
    /// If a provider with the same name exists, it will be replaced.
    ///
    /// # Arguments
    /// * `name` - Unique name for this authentication provider
    /// * `auth` - The authenticator implementation
    ///
    /// # Example
    /// ```rust
    /// use caller::{AuthRegistry, BearerAuth};
    ///
    /// AuthRegistry::register("bearer", BearerAuth::new("my-token".to_string())).unwrap();
    /// # AuthRegistry::clear().unwrap();
    /// ```
    pub fn register(name: &str, auth: impl Authenticator + 'static) -> Result<(), CallerError> {
        let provider = AuthProvider::from_trait(auth);
        let mut providers = AuthRegistry::global()
            .providers
            .write()
            .map_err(|_| CallerError::lock_poisoned("global auth registry"))?;
        providers.insert(name.to_string(), provider);
        Ok(())
    }

    /// Register a closure-based authentication provider
    ///
    /// This is useful for one-off custom authentication that doesn't need
    /// to implement the full Authenticator trait.
    ///
    /// # Arguments
    /// * `name` - Unique name for this authentication provider
    /// * `f` - Closure that takes RequestBuilder and AuthContext, returns modified builder
    ///
    /// # Example
    /// ```rust
    /// use caller::AuthRegistry;
    ///
    /// AuthRegistry::register_closure("custom", |builder, ctx| async move {
    ///     Ok(builder.header("X-Custom-Auth", "value"))
    /// }).unwrap();
    /// ```
    pub fn register_closure<F, Fut>(name: &str, f: F) -> Result<(), CallerError>
    where
        F: Fn(reqwest::RequestBuilder, &AuthContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<reqwest::RequestBuilder, CallerError>>
            + Send
            + 'static,
    {
        let provider = AuthProvider::from_closure(f);
        let mut providers = AuthRegistry::global()
            .providers
            .write()
            .map_err(|_| CallerError::lock_poisoned("global auth registry"))?;
        providers.insert(name.to_string(), provider);
        Ok(())
    }

    /// Get an authentication provider by name
    ///
    /// Returns a cloned Arc reference to the provider for thread-safe use.
    ///
    /// # Arguments
    /// * `name` - name of the authentication provider
    ///
    /// # Returns
    /// Some(AuthProvider) if found, None otherwise
    pub fn get(name: &str) -> Option<AuthProvider> {
        let providers = AuthRegistry::global().providers.read().ok()?;
        providers.get(name).cloned()
    }

    /// Check if a provider with the given name exists
    pub fn contains(name: &str) -> bool {
        if let Ok(providers) = AuthRegistry::global().providers.read() {
            providers.contains_key(name)
        } else {
            false
        }
    }

    /// Remove a provider by name
    ///
    /// # Arguments
    /// * `name` - name of the provider to remove
    ///
    /// # Returns
    /// true if the provider was removed, false if it didn't exist
    pub fn remove(name: &str) -> Result<bool, CallerError> {
        let mut providers = AuthRegistry::global()
            .providers
            .write()
            .map_err(|_| CallerError::lock_poisoned("global auth registry"))?;
        Ok(providers.remove(name).is_some())
    }

    /// Update an existing authenticator (same as register, but semantically clear)
    ///
    /// This replaces the authenticator if it exists, or adds it if it doesn't.
    ///
    /// # Arguments
    /// * `name` - name of the authentication provider
    /// * `auth` - New authenticator implementation
    pub fn update(name: &str, auth: impl Authenticator + 'static) -> Result<(), CallerError> {
        Self::register(name, auth)
    }

    /// Update a closure-based authentication provider
    pub fn update_closure<F, Fut>(name: &str, f: F) -> Result<(), CallerError>
    where
        F: Fn(reqwest::RequestBuilder, &AuthContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<reqwest::RequestBuilder, CallerError>>
            + Send
            + 'static,
    {
        Self::register_closure(name, f)
    }

    /// Clear all registered providers
    pub fn clear() -> Result<(), CallerError> {
        let mut providers = AuthRegistry::global()
            .providers
            .write()
            .map_err(|_| CallerError::lock_poisoned("global auth registry"))?;
        providers.clear();
        Ok(())
    }

    /// Get all registered provider names
    pub fn list() -> Result<Vec<String>, CallerError> {
        let providers = AuthRegistry::global()
            .providers
            .read()
            .map_err(|_| CallerError::lock_poisoned("global auth registry"))?;
        Ok(providers.keys().cloned().collect())
    }

    /// Get the number of registered providers
    pub fn count() -> Result<usize, CallerError> {
        let providers = AuthRegistry::global()
            .providers
            .read()
            .map_err(|_| CallerError::lock_poisoned("global auth registry"))?;
        Ok(providers.len())
    }

    pub(crate) fn snapshot() -> Result<HashMap<String, AuthProvider>, CallerError> {
        let providers = AuthRegistry::global()
            .providers
            .read()
            .map_err(|_| CallerError::lock_poisoned("global auth registry"))?;
        Ok(providers.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use reqwest::RequestBuilder;

    struct TestAuth;

    #[async_trait]
    impl Authenticator for TestAuth {
        async fn authenticate(
            &self,
            builder: RequestBuilder,
            _context: &AuthContext,
        ) -> Result<RequestBuilder, CallerError> {
            Ok(builder.header("X-Test", "test"))
        }
    }

    #[test]
    fn test_register_and_get() {
        // Use unique name to avoid conflicts with parallel tests
        let name = format!("test_register_and_get_{}", std::process::id());
        AuthRegistry::register(&name, TestAuth).unwrap();
        assert!(AuthRegistry::contains(&name));
        assert!(AuthRegistry::get(&name).is_some());
        AuthRegistry::remove(&name).ok();
    }

    #[test]
    fn test_remove() {
        let name = format!("test_remove_{}", std::process::id());
        AuthRegistry::register(&name, TestAuth).unwrap();
        assert!(AuthRegistry::remove(&name).unwrap());
        assert!(!AuthRegistry::contains(&name));
    }

    #[test]
    fn test_list() {
        let name1 = format!("test_list_1_{}", std::process::id());
        let name2 = format!("test_list_2_{}", std::process::id());
        AuthRegistry::register(&name1, TestAuth).unwrap();
        AuthRegistry::register(&name2, TestAuth).unwrap();
        let list = AuthRegistry::list().unwrap();
        assert!(list.contains(&name1));
        assert!(list.contains(&name2));
        AuthRegistry::remove(&name1).ok();
        AuthRegistry::remove(&name2).ok();
    }
}
