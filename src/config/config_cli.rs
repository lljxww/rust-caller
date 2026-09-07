//! Interactive CLI for configuration management

use crate::config::{ConfigBuilder, ConfigFormat};
use crate::domain::api_config::{ApiConfig, HttpMethod};
use std::io::{self, Write};

/// Run the configuration management CLI
pub(crate) fn run_interactive() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Caller Configuration Manager");
    println!("================================\n");

    let mut builder = ConfigBuilder::new();

    loop {
        println!("\n📋 Menu:");
        println!("  1. Add Service");
        println!("  2. List Services");
        println!("  3. Edit Service");
        println!("  4. Remove Service");
        println!("  5. Add API to Service");
        println!("  6. View Configuration");
        println!("  7. Export Configuration");
        println!("  8. Load Configuration");
        println!("  9. Convert Format");
        println!("  0. Exit");
        println!();

        let choice = read_line("Select option: ")?;

        match choice.trim() {
            "1" => add_service(&mut builder)?,
            "2" => list_services(&builder)?,
            "3" => edit_service(&mut builder)?,
            "4" => remove_service(&mut builder)?,
            "5" => add_api(&mut builder)?,
            "6" => view_config(&builder)?,
            "7" => export_config(&builder)?,
            "8" => load_config(&mut builder)?,
            "9" => convert_config()?,
            "0" => {
                println!("👋 Goodbye!");
                break;
            }
            _ => println!("❌ Invalid option"),
        }
    }

    Ok(())
}

fn add_service(builder: &mut ConfigBuilder) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n➕ Add New Service\n");

    let name = read_line_required("Service name: ")?;
    let base_url = read_line_required("Base URL (e.g., https://api.example.com): ")?;
    let auth = read_line("AuthConfig type (optional): ")?;
    let timeout = read_line("timeout in ms (default: 30000): ")?;

    let mut svc_builder = builder.service(&name, &base_url);

    if !auth.trim().is_empty() {
        svc_builder = svc_builder.auth(auth.trim());
    }

    if let Ok(t) = timeout.trim().parse::<u32>() {
        svc_builder = svc_builder.timeout(t);
    } else {
        svc_builder = svc_builder.timeout(30000);
    }

    // Ask if user wants to add APIs now
    loop {
        let add_api = read_line("Add API endpoint? (y/n): ")?;
        if add_api.trim().to_lowercase() != "y" {
            break;
        }

        let method = read_line_required("API method name: ")?;
        let url = read_line_required("URL path (e.g., /users/{id}): ")?;
        let http_method = read_line_default(
            "HTTP method (GET/POST/PUT/DELETE/PATCH/HEAD/OPTIONS)",
            "GET",
        )?;
        let param_type = read_line_required("Param type (none/query/path/json/form): ")?;
        let description = read_line("Description (optional): ")?;

        let api = ApiConfig {
            method,
            url,
            http_method: HttpMethod::parse(&http_method)?,
            param_type: ApiConfig::parse_param_types(&param_type)?,
            description: if description.trim().is_empty() {
                None
            } else {
                Some(description)
            },
            need_cache: None,
            cache_time: None,
            content_type: None,
            authorization_type: None,
            timeout: None,
            use_new_http_client: None,
        };

        svc_builder = svc_builder.api_full(api);
    }

    svc_builder.build();
    println!("✅ Service '{}' added successfully!", name);

    Ok(())
}

fn list_services(builder: &ConfigBuilder) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Services:\n");

    let services = builder.list_services();
    if services.is_empty() {
        println!("  (no services defined)");
        return Ok(());
    }

    for name in services {
        if let Some(svc) = builder.get_service(name) {
            println!("  📁 {} - {}", svc.api_name, svc.base_url);
            println!("     APIs: {}", svc.api_items.len());
            for api in &svc.api_items {
                println!(
                    "       • {} [{}] {}",
                    api.method,
                    api.http_method.as_str(),
                    api.url
                );
            }
        }
    }

    Ok(())
}

fn edit_service(builder: &mut ConfigBuilder) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n✏️ Edit Service\n");

    let name = read_line_required("Service name to edit: ")?;

    // Clone the service data to avoid borrow conflicts
    let service_data = builder.get_service(&name).cloned();

    if service_data.is_none() {
        println!("❌ Service '{}' not found", name);
        return Ok(());
    }

    let service = service_data.unwrap();
    println!("Current base URL: {}", service.base_url);

    let new_url = read_line(&format!(
        "New base URL (enter to keep '{}'): ",
        service.base_url
    ))?;
    let base_url = if new_url.trim().is_empty() {
        service.base_url.clone()
    } else {
        new_url
    };

    let current_auth = service.authorization_type.as_deref().unwrap_or("(none)");
    let new_auth = read_line(&format!(
        "New auth type (current: {}, enter to keep): ",
        current_auth
    ))?;

    let current_timeout = service.timeout.unwrap_or(30000);
    let new_timeout = read_line(&format!(
        "New timeout (current: {}ms, enter to keep): ",
        current_timeout
    ))?;

    // Rebuild service with updated values
    let mut svc_builder = builder.service(&name, &base_url);

    if !new_auth.trim().is_empty() {
        svc_builder = svc_builder.auth(&new_auth);
    } else if let Some(ref auth) = service.authorization_type {
        svc_builder = svc_builder.auth(auth);
    }

    if let Ok(t) = new_timeout.trim().parse::<u32>() {
        svc_builder = svc_builder.timeout(t);
    } else {
        svc_builder = svc_builder.timeout(current_timeout);
    }

    // Copy existing APIs
    for api in &service.api_items {
        svc_builder = svc_builder.api_full(api.clone());
    }

    svc_builder.build();
    println!("✅ Service '{}' updated!", name);

    Ok(())
}

fn remove_service(builder: &mut ConfigBuilder) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🗑️ Remove Service\n");

    let name = read_line_required("Service name to remove: ")?;

    if builder.remove_service(&name) {
        println!("✅ Service '{}' removed", name);
    } else {
        println!("❌ Service '{}' not found", name);
    }

    Ok(())
}

fn add_api(builder: &mut ConfigBuilder) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n➕ Add API Endpoint\n");

    let service_name = read_line_required("Service name: ")?;

    if builder.get_service(&service_name).is_none() {
        println!("❌ Service '{}' not found", service_name);
        return Ok(());
    }

    let method = read_line_required("API method name: ")?;
    let url = read_line_required("URL path: ")?;
    let http_method = read_line_default("HTTP method", "GET")?;
    let param_type = read_line_required("Param type: ")?;
    let description = read_line("Description: ")?;

    let api = ApiConfig {
        method,
        url,
        http_method: HttpMethod::parse(&http_method)?,
        param_type: ApiConfig::parse_param_types(&param_type)?,
        description: if description.trim().is_empty() {
            None
        } else {
            Some(description)
        },
        need_cache: None,
        cache_time: None,
        content_type: None,
        authorization_type: None,
        timeout: None,
        use_new_http_client: None,
    };

    builder.add_api(&service_name, api)?;
    println!("✅ API added to service '{}'", service_name);

    Ok(())
}

fn view_config(builder: &ConfigBuilder) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📄 Current Configuration\n");

    println!("Select output format:");
    println!("  1. JSON");
    println!("  2. YAML");
    println!("  3. TOML");

    let choice = read_line("Format (1-3, default: 1): ")?;
    let format = match choice.trim() {
        "2" => ConfigFormat::Yaml,
        "3" => ConfigFormat::Toml,
        _ => ConfigFormat::Json,
    };

    let content = builder.to_format(format)?;
    println!("\n{}", content);

    Ok(())
}

fn export_config(builder: &ConfigBuilder) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n💾 Export Configuration\n");

    let path = read_line_required("Output file path (e.g., config.json): ")?;

    builder.save(&path)?;
    println!("✅ Configuration saved to '{}'", path);

    Ok(())
}

fn load_config(builder: &mut ConfigBuilder) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📂 Load Configuration\n");

    let path = read_line_required("Input file path: ")?;

    *builder = ConfigBuilder::load(&path)?;
    println!("✅ Configuration loaded from '{}'", path);

    Ok(())
}

fn convert_config() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Convert Configuration Format\n");

    let input = read_line_required("Input file path: ")?;
    let output = read_line_required("Output file path: ")?;

    ConfigBuilder::convert(&input, &output)?;
    println!("✅ Converted '{}' to '{}'", input, output);

    Ok(())
}

// Helper functions

fn read_line(prompt: &str) -> Result<String, io::Error> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn read_line_required(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    loop {
        let input = read_line(prompt)?;
        if !input.is_empty() {
            return Ok(input);
        }
        println!("❌ This field is required");
    }
}

fn read_line_default(prompt: &str, default: &str) -> Result<String, io::Error> {
    let input = read_line(&format!("{} [{}]: ", prompt, default))?;
    Ok(if input.is_empty() {
        default.to_string()
    } else {
        input
    })
}
