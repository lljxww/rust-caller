//! OpenAPI specification generation for Caller
//!
//! This module provides functionality to generate OpenAPI 3.0 specifications
//! from Caller configuration files.

mod generator;
mod types;

pub use generator::OpenApiGenerator;
pub use types::*;
