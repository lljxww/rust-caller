use caller::CallerError;
use caller::config::config_loader::{ConfigFormat, ConfigLoader};
use std::fs;

#[test]
fn test_detect_format_from_extension() {
    assert_eq!(
        ConfigFormat::from_extension("json"),
        Some(ConfigFormat::Json)
    );
    assert_eq!(
        ConfigFormat::from_extension("yaml"),
        Some(ConfigFormat::Yaml)
    );
    assert_eq!(
        ConfigFormat::from_extension("yml"),
        Some(ConfigFormat::Yaml)
    );
    assert_eq!(
        ConfigFormat::from_extension("toml"),
        Some(ConfigFormat::Toml)
    );
    assert_eq!(ConfigFormat::from_extension("txt"), None);
    assert_eq!(ConfigFormat::from_extension("xml"), None);
}

#[test]
fn test_detect_format_from_path() {
    assert_eq!(
        ConfigFormat::detect_from_path("./config.json"),
        Some(ConfigFormat::Json)
    );
    assert_eq!(
        ConfigFormat::detect_from_path("./config.yaml"),
        Some(ConfigFormat::Yaml)
    );
    assert_eq!(
        ConfigFormat::detect_from_path("./config.yml"),
        Some(ConfigFormat::Yaml)
    );
    assert_eq!(
        ConfigFormat::detect_from_path("./config.toml"),
        Some(ConfigFormat::Toml)
    );
    assert_eq!(ConfigFormat::detect_from_path("./config.txt"), None);
    assert_eq!(ConfigFormat::detect_from_path("./config"), None);
}

#[test]
fn test_convert_with_explicit_format() {
    let input_file = "test_explicit_input.json";
    let output_file = "test_explicit_output.config";

    // Create a test JSON config
    let json_content = r#"{
      "authorizations": [],
      "service_items": [
        {
          "api_name": "TestService",
          "base_url": "https://example.com",
          "api_items": []
        }
      ]
    }"#;
    fs::write(input_file, json_content).unwrap();

    // Convert with explicit format (ignore output file extension)
    let result =
        ConfigLoader::convert_config_with_format(input_file, output_file, ConfigFormat::Yaml);
    assert!(
        result.is_ok(),
        "Should convert with explicit format successfully"
    );

    // Verify output file exists
    assert!(
        fs::metadata(output_file).is_ok(),
        "Output file should exist"
    );

    // Verify that content is YAML format
    let output_content = fs::read_to_string(output_file).unwrap();
    assert!(
        output_content.contains("authorizations:"),
        "Output should contain YAML format"
    );

    // Cleanup
    fs::remove_file(input_file).ok();
    fs::remove_file(output_file).ok();
}

#[test]
fn test_convert_preserves_data() {
    let input_file = "test_preserve_input.json";
    let output_file = "test_preserve_output.yaml";

    // Create a detailed test JSON config
    let json_content = r#"{
      "authorizations": [
        {
          "name": "TestAuth",
          "authorization_info": "Bearer token123"
        }
      ],
      "service_items": [
        {
          "api_name": "GitHub_API",
          "base_url": "https://api.github.com",
          "timeout": 10000,
          "api_items": [
            {
              "method": "get_user",
              "url": "/users/{username}",
              "http_method": "GET",
              "param_type": "path",
              "description": "Get user info"
            }
          ]
        }
      ]
    }"#;
    fs::write(input_file, json_content).unwrap();

    // Convert JSON to YAML
    ConfigLoader::convert_config(input_file, output_file).unwrap();

    // Load both and compare
    let original_config = ConfigLoader::load_config_from_path(input_file).unwrap();
    let converted_config = ConfigLoader::load_config_from_path(output_file).unwrap();

    assert_eq!(
        original_config.authorizations.len(),
        converted_config.authorizations.len(),
        "Should preserve number of authorizations"
    );
    assert_eq!(
        original_config.service_items.len(),
        converted_config.service_items.len(),
        "Should preserve number of service items"
    );
    assert_eq!(
        original_config.service_items[0].api_name, converted_config.service_items[0].api_name,
        "Should preserve service name"
    );
    assert_eq!(
        original_config.service_items[0].api_items.len(),
        converted_config.service_items[0].api_items.len(),
        "Should preserve number of API items"
    );

    // Cleanup
    fs::remove_file(input_file).ok();
    fs::remove_file(output_file).ok();
}

#[test]
fn test_load_invalid_json() {
    let test_file = "test_invalid.json";
    fs::write(test_file, "{ invalid json }").unwrap();

    let result = ConfigLoader::load_config_from_path(test_file);
    assert!(result.is_err(), "Should fail to load invalid JSON");
    assert!(matches!(
        result.unwrap_err(),
        CallerError::ConfigParseError { format, .. } if format == "json"
    ));

    // Cleanup
    fs::remove_file(test_file).ok();
}

#[test]
fn test_load_invalid_yaml() {
    let test_file = "test_invalid.yaml";
    fs::write(test_file, "{ invalid: yaml: content }").unwrap();

    let result = ConfigLoader::load_config_from_path(test_file);
    assert!(result.is_err(), "Should fail to load invalid YAML");
    assert!(matches!(
        result.unwrap_err(),
        CallerError::ConfigParseError { format, .. } if format == "yaml"
    ));

    // Cleanup
    fs::remove_file(test_file).ok();
}

#[test]
fn test_load_invalid_toml() {
    let test_file = "test_invalid.toml";
    fs::write(test_file, "invalid toml content [unclosed").unwrap();

    let result = ConfigLoader::load_config_from_path(test_file);
    assert!(result.is_err(), "Should fail to load invalid TOML");
    assert!(matches!(
        result.unwrap_err(),
        CallerError::ConfigParseError { format, .. } if format == "toml"
    ));

    // Cleanup
    fs::remove_file(test_file).ok();
}

#[test]
fn test_convert_json_to_yaml() {
    let input_file = "test_json2yaml_input.json";
    let output_file = "test_json2yaml_output.yaml";

    // Create a test JSON config
    let json_content = r#"{
      "authorizations": [],
      "service_items": [
        {
          "api_name": "TestService",
          "base_url": "https://example.com",
          "api_items": []
        }
      ]
    }"#;
    fs::write(input_file, json_content).unwrap();

    // Convert JSON to YAML
    let result = ConfigLoader::convert_config(input_file, output_file);
    assert!(result.is_ok(), "Should convert JSON to YAML successfully");

    // Verify output file exists and is valid YAML
    assert!(
        fs::metadata(output_file).is_ok(),
        "Output YAML file should exist"
    );

    // Verify that converted YAML can be loaded
    let yaml_result = ConfigLoader::load_config_from_path(output_file);
    assert!(yaml_result.is_ok(), "Converted YAML should be valid");

    // Cleanup
    fs::remove_file(input_file).ok();
    fs::remove_file(output_file).ok();
}

#[test]
fn test_convert_yaml_to_toml() {
    let input_file = "test_yaml2toml_input.yaml";
    let output_file = "test_yaml2toml_output.toml";

    // Create a test YAML config
    let yaml_content = r#"authorizations: []
service_items:
  - api_name: TestService
    base_url: https://example.com
    api_items: []
"#;
    fs::write(input_file, yaml_content).unwrap();

    // Convert YAML to TOML
    let result = ConfigLoader::convert_config(input_file, output_file);
    assert!(result.is_ok(), "Should convert YAML to TOML successfully");

    // Verify output file exists and is valid TOML
    assert!(
        fs::metadata(output_file).is_ok(),
        "Output TOML file should exist"
    );

    // Verify that converted TOML can be loaded
    let toml_result = ConfigLoader::load_config_from_path(output_file);
    assert!(toml_result.is_ok(), "Converted TOML should be valid");

    // Cleanup
    fs::remove_file(input_file).ok();
    fs::remove_file(output_file).ok();
}

#[test]
fn test_convert_toml_to_json() {
    let input_file = "test_toml2json_input.toml";
    let output_file = "test_toml2json_output.json";

    // Create a test TOML config
    let toml_content = r#"authorizations = []

[[service_items]]
api_name = "TestService"
base_url = "https://example.com"
api_items = []
"#;
    fs::write(input_file, toml_content).unwrap();

    // Convert TOML to JSON
    let result = ConfigLoader::convert_config(input_file, output_file);
    assert!(result.is_ok(), "Should convert TOML to JSON successfully");

    // Verify output file exists and is valid JSON
    assert!(
        fs::metadata(output_file).is_ok(),
        "Output JSON file should exist"
    );

    // Verify that converted JSON can be loaded
    let json_result = ConfigLoader::load_config_from_path(output_file);
    assert!(json_result.is_ok(), "Converted JSON should be valid");

    // Cleanup
    fs::remove_file(input_file).ok();
    fs::remove_file(output_file).ok();
}

#[test]
fn test_load_json_config() {
    let result = ConfigLoader::load_config_from_path("samples/api_config_example.json");
    if let Err(e) = &result {
        eprintln!("JSON config loading error: {}", e);
    }
    assert!(result.is_ok(), "Should load JSON config successfully");

    let config = result.unwrap();
    assert!(
        !config.service_items.is_empty(),
        "Should have service items"
    );
}

#[test]
fn test_load_yaml_config() {
    let result = ConfigLoader::load_config_from_path("samples/api_config_example.yaml");
    assert!(result.is_ok(), "Should load YAML config successfully");

    let config = result.unwrap();
    assert!(
        !config.service_items.is_empty(),
        "Should have service items"
    );
}

#[test]
fn test_load_toml_config() {
    let result = ConfigLoader::load_config_from_path("samples/api_config_example.toml");
    assert!(result.is_ok(), "Should load TOML config successfully");

    let config = result.unwrap();
    assert!(
        !config.service_items.is_empty(),
        "Should have service items"
    );
}

#[test]
fn test_json_yaml_toml_config_consistency() {
    let json_result = ConfigLoader::load_config_from_path("samples/api_config_example.json");
    let yaml_result = ConfigLoader::load_config_from_path("samples/api_config_example.yaml");
    let toml_result = ConfigLoader::load_config_from_path("samples/api_config_example.toml");

    assert!(json_result.is_ok(), "JSON config should be valid");
    assert!(yaml_result.is_ok(), "YAML config should be valid");
    assert!(toml_result.is_ok(), "TOML config should be valid");

    let json_config = json_result.unwrap();
    let yaml_config = yaml_result.unwrap();
    let toml_config = toml_result.unwrap();

    // Check that all formats load the same number of services
    assert_eq!(
        json_config.service_items.len(),
        yaml_config.service_items.len(),
        "JSON and YAML should have same number of services"
    );
    assert_eq!(
        json_config.service_items.len(),
        toml_config.service_items.len(),
        "JSON and TOML should have same number of services"
    );

    // Check that the first service is the same across all formats
    if let Some(json_service) = json_config.service_items.first() {
        let yaml_service = yaml_config.service_items.first().unwrap();
        let toml_service = toml_config.service_items.first().unwrap();

        assert_eq!(json_service.api_name, yaml_service.api_name);
        assert_eq!(json_service.api_name, toml_service.api_name);
        assert_eq!(json_service.base_url, yaml_service.base_url);
        assert_eq!(json_service.base_url, toml_service.base_url);
    }
}

#[test]
fn test_load_config_with_explicit_format() {
    // Test loading with explicit format specification
    let json_result = ConfigLoader::load_config_from_path_with_format(
        "samples/api_config_example.json",
        ConfigFormat::Json,
    );
    assert!(json_result.is_ok(), "Should load JSON with explicit format");

    let yaml_result = ConfigLoader::load_config_from_path_with_format(
        "samples/api_config_example.yaml",
        ConfigFormat::Yaml,
    );
    assert!(yaml_result.is_ok(), "Should load YAML with explicit format");

    let toml_result = ConfigLoader::load_config_from_path_with_format(
        "samples/api_config_example.toml",
        ConfigFormat::Toml,
    );
    assert!(toml_result.is_ok(), "Should load TOML with explicit format");
}

#[test]
fn test_load_unsupported_format() {
    // Create a temporary file with unsupported extension
    let test_file = "test_config.txt";
    fs::write(test_file, "{ \"test\": \"data\" }").unwrap();

    let result = ConfigLoader::load_config_from_path(test_file);
    assert!(result.is_err(), "Should fail to load unsupported format");
    assert!(matches!(
        result.unwrap_err(),
        CallerError::UnsupportedConfigFormat { path } if path == test_file
    ));

    // Cleanup
    fs::remove_file(test_file).ok();
}

#[test]
fn test_load_missing_file_returns_structured_error() {
    let result = ConfigLoader::load_config_from_path("missing_config_for_test.json");

    assert!(matches!(
        result.unwrap_err(),
        CallerError::ConfigFileNotFound { path } if path == "missing_config_for_test.json"
    ));
}
