pub(crate) mod config_builder;
pub(crate) mod config_cli;
pub(crate) mod config_loader;
mod format;

pub use config_builder::{ApiEndpointBuilder, ConfigBuilder, ServiceBuilder};
pub use config_loader::ConfigLoader;
pub use format::ConfigFormat;

/// Run the interactive configuration CLI
pub fn run_config_cli() -> Result<(), Box<dyn std::error::Error>> {
    config_cli::run_interactive()
}
