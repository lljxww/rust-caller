//! Configuration Manager CLI
//!
//! Run with: cargo run --example config-cli
//!
//! Features:
//! - Create and edit configuration files
//! - Add/remove services and APIs
//! - Export to JSON/YAML/TOML
//! - Convert between formats

fn main() {
    println!("🔧 Caller Configuration Manager");
    println!("================================\n");
    
    // Use the config CLI module
    caller::config::config_cli::run_interactive().unwrap_or_else(|e| {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    });
}
