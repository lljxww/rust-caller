use std::path::Path;

/// Configuration serialization format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// JSON configuration.
    Json,
    /// YAML configuration (`.yaml` or `.yml`).
    Yaml,
    /// TOML configuration.
    Toml,
}

impl ConfigFormat {
    /// Detect a format from a file extension without a leading dot.
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_ascii_lowercase().as_str() {
            "json" => Some(Self::Json),
            "yaml" | "yml" => Some(Self::Yaml),
            "toml" => Some(Self::Toml),
            _ => None,
        }
    }

    /// Detect a format from a path's final extension.
    pub fn detect_from_path(path: impl AsRef<Path>) -> Option<Self> {
        path.as_ref()
            .extension()
            .and_then(|extension| extension.to_str())
            .and_then(Self::from_extension)
    }

    /// Return the canonical extension for this format.
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Toml => "toml",
        }
    }

    /// Return all supported configuration formats.
    pub const fn all() -> &'static [Self] {
        &[Self::Json, Self::Yaml, Self::Toml]
    }
}
