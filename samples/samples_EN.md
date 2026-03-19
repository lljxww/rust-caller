[English](samples_EN.md) | [简体中文](samples_CN.md)

# Caller Example Code

This directory contains various usage examples of the Caller library to help developers get started quickly and understand the library's features.

## Example List

### 0. Configuration File Examples

I've created multiple JSON configuration file examples that you can choose from based on your project needs:

#### [`api_config_example.json`](./api_config_example.json) - Complete Configuration Example
The most detailed configuration file example, including:
- Multiple authentication methods (Bearer, Basic, Header, Query)
- 6 different API services (GitHub, Weather, News, Translation, Docker Registry, Mock API)
- Complete parameter configuration, cache settings, timeout settings
- Global configuration items (retry, rate limiting, circuit breaker, etc.)

#### [`minimal_config_example.json`](./minimal_config_example.json) - Minimal Configuration Example
Concise configuration for quick start:
- Basic authentication configuration
- 3 common API endpoints
- Environment variable support

#### [`ecommerce_config_example.json`](./ecommerce_config_example.json) - E-commerce Configuration Example
Configuration designed for e-commerce projects:
- Shopify store management API
- Alipay/WeChat Pay API
- JD product search API
- Inventory management API
- Logistics and delivery API

### 1. [`basic_usage.rs`](./basic_usage.rs)
**Basic Usage Example**

Demonstrates basic Caller functionality:
- API calls without parameters
- Path parameter usage
- Query parameter usage
- JSON request body
- Deep path access

```bash
cargo run --example basic_usage
```

**Learning Points:**
- Understand basic usage of `call()` function
- Master usage of different parameter types
- Learn basic operations of `ApiResult`

### 2. [`advanced_usage.rs`](./advanced_usage.rs)
**Advanced Usage Example**

Demonstrates advanced Caller features:
- Different HTTP methods (GET, POST, PUT, PATCH)
- Complex JSON data operations
- Array traversal and access
- Hierarchical data access
- Batch operation examples

```bash
cargo run --example advanced_usage
```

**Learning Points:**
- Understand complete CRUD operation flow
- Master handling of complex data structures
- Learn bulk data traversal

### 3. [`error_handling.rs`](./error_handling.rs)
**Error Handling Example**

Demonstrates handling of various error scenarios:
- Service not found errors
- Method not found errors
- Parameter missing errors
- HTTP network errors
- Error retry mechanism
- Safe error handling functions

```bash
cargo run --example error_handling
```

**Learning Points:**
- Error type identification and handling
- Safe API calling patterns
- Implementation of retry mechanisms

### 4. [`config_management.rs`](./config_management.rs)
**Configuration Management Example**

Demonstrates various configuration management usages:
- Multi-environment configuration file creation
- Configuration validation and checking
- Environment variable integration
- Hot reload strategy
- Configuration file comparison

```bash
cargo run --example config_management
```

**Learning Points:**
- Understand configuration file structure
- Multi-environment configuration management
- Configuration validation and security

## How to Run Examples

### Prerequisites
Make sure you've compiled the Caller library:

```bash
cargo build
```

### Run Specific Examples
```bash
cargo run --example basic_usage
cargo run --example advanced_usage
cargo run --example error_handling
cargo run --example config_management
```

### Run All Examples
```bash
for example in basic_usage advanced_usage error_handling config_management; do
    echo "=== Running example: $example ==="
    cargo run --example $example
    echo
done
```

## Skills After Completing These Examples

By running these examples, you will be able to:

### Basic Skills
- ✅ Properly configure and use the Caller library
- ✅ Call various types of API endpoints
- ✅ Handle path parameters, query parameters, JSON parameters
- ✅ Parse and access JSON response data

### Intermediate Skills
- ✅ Implement complete CRUD operations
- ✅ Handle complex nested data structures
- ✅ Batch data operations and processing
- ✅ Error handling and retry mechanisms

### Advanced Skills
- ✅ Multi-environment configuration management
- ✅ Configuration file validation and security
- ✅ Hot reload strategy implementation
- ✅ Enterprise-level error handling patterns

## Configuration File Requirements

Before running the examples, make sure the project root directory has the following configuration files:

1. **caller.json** - Main API configuration file
   - Defines available API endpoints
   - Authentication configuration
   - Service configuration

### Quick Start

You can choose to use the provided example configuration files:

#### Use Example Configuration Files
```bash
# Copy minimal configuration (suitable for quick testing)
cp samples/minimal_config_example.json caller.json

# Or use complete configuration (most feature-rich)
cp samples/api_config_example.json caller.json

# Or use e-commerce configuration (if doing e-commerce projects)
cp samples/ecommerce_config_example.json caller.json
```

#### Custom Configuration
If you need your own configuration file:

1. Refer to example configuration files in the `samples/` directory
2. Modify the `caller.json` file according to actual API requirements
3. Run the `config_management` example to see configuration validation details

The `config_management` example will automatically create example configuration files, while other examples require `caller.json` to be properly configured.

## Custom Examples

You can create your own usage examples based on these:

1. Copy existing example files
2. Modify to use your actual API endpoints
3. Adjust parameters and data processing logic according to business requirements
4. Add more error handling and edge case testing

## Performance Optimization Recommendations

1. **Cache Strategy**: Enable caching for frequently called APIs
2. **Connection Pool**: Reuse HTTP connections to reduce connection overhead
3. **Batch Operations**: Use batch APIs to reduce network request frequency
4. **Error Retry**: Implement reasonable retry strategies
5. **Timeout Settings**: Set appropriate timeout times for each API

## FAQ

### Q: What if the example fails to run?
A: First check if the `caller.json` configuration file is correct, then check the specific error message.

### Q: How to add my own API?
A: Add new ServiceItems and corresponding ApiItems in `caller.json`.

### Q: How to debug network issues?
A: Check the `ApiResult.raw` field to get the original response, check network connection and API endpoint accessibility.

### Q: How to handle authentication failures?
A: Check Authorizations configuration, ensure Token and authentication type are correct.

## Contributing

Welcome to submit more usage examples! Please ensure:
- Example code has detailed comments
- Includes complete error handling
- Code style follows Rust standard format
- Provides running instructions and learning points