pub mod config_builder;
pub mod config_cli;
pub mod config_loader;

pub use config_builder::{ConfigBuilder, ConfigFormat, ServiceBuilder};
pub use config_loader::ConfigLoader;

/// Run the interactive configuration CLI
pub fn run_config_cli() -> Result<(), Box<dyn std::error::Error>> {
    config_cli::run_interactive()
}

