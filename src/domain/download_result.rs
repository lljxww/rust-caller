use crate::shared::error::CallerError;
use reqwest::StatusCode;
use std::path::Path;

/// Download result containing file information and content
#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub status_code: StatusCode,
    pub content: Vec<u8>,
    pub content_type: String,
    pub file_extension: String,
    pub suggested_filename: Option<String>,
}

impl DownloadResult {
    /// Create a new download result from HTTP response
    pub fn from_response(
        status_code: StatusCode,
        content: Vec<u8>,
        content_type: Option<String>,
    ) -> Result<DownloadResult, CallerError> {
        let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_string());
        
        // Auto-detect file extension from content type
        let file_extension = Self::detect_extension_from_mime(&content_type);
        
        Ok(DownloadResult {
            status_code,
            content,
            content_type,
            file_extension,
            suggested_filename: None,
        })
    }

    /// Detect file extension from MIME type
    fn detect_extension_from_mime(mime: &str) -> String {
        let mime_lower = mime.to_lowercase();
        
        // Remove charset and other parameters
        let mime_base = mime_lower.split(';').next().unwrap_or(&mime_lower).trim();
        
        match mime_base {
            "text/html" => "html".to_string(),
            "text/plain" => "txt".to_string(),
            "text/xml" => "xml".to_string(),
            "text/css" => "css".to_string(),
            "text/javascript" | "application/javascript" => "js".to_string(),
            "application/json" => "json".to_string(),
            "application/pdf" => "pdf".to_string(),
            "application/zip" => "zip".to_string(),
            "application/x-rar-compressed" => "rar".to_string(),
            "application/x-7z-compressed" => "7z".to_string(),
            "application/x-tar" => "tar".to_string(),
            "application/x-gtar" => "tgz".to_string(),
            "application/x-gzip" => "gz".to_string(),
            "application/x-bzip2" => "bz2".to_string(),
            "application/x-shockwave-flash" => "swf".to_string(),
            "application/msword" => "doc".to_string(),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => "docx".to_string(),
            "application/vnd.ms-excel" => "xls".to_string(),
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => "xlsx".to_string(),
            "application/vnd.ms-powerpoint" => "ppt".to_string(),
            "application/vnd.openxmlformats-officedocument.presentationml.presentation" => "pptx".to_string(),
            "application/vnd.ms-fontobject" => "eot".to_string(),
            "application/font-woff" => "woff".to_string(),
            "application/font-woff2" => "woff2".to_string(),
            "application/octet-stream" => "bin".to_string(),
            "image/jpeg" => "jpg".to_string(),
            "image/png" => "png".to_string(),
            "image/gif" => "gif".to_string(),
            "image/bmp" => "bmp".to_string(),
            "image/webp" => "webp".to_string(),
            "image/svg+xml" => "svg".to_string(),
            "image/tiff" => "tiff".to_string(),
            "image/x-icon" => "ico".to_string(),
            "video/mp4" => "mp4".to_string(),
            "video/webm" => "webm".to_string(),
            "video/quicktime" => "mov".to_string(),
            "video/x-msvideo" => "avi".to_string(),
            "video/mpeg" => "mpeg".to_string(),
            "audio/mpeg" => "mp3".to_string(),
            "audio/wav" => "wav".to_string(),
            "audio/ogg" => "ogg".to_string(),
            "audio/webm" => "weba".to_string(),
            _ => {
                // Try to extract extension from URL-like content types
                if mime_base.contains("+") {
                    // Handle types like "application/vnd.api+json"
                    if let Some(ext) = mime_base.rsplit('+').next() {
                        return ext.to_string();
                    }
                }
                "bin".to_string()
            }
        }
    }

    /// Override the auto-detected file extension with a custom one
    pub fn with_extension(mut self, extension: &str) -> Self {
        let ext = if extension.starts_with('.') {
            extension[1..].to_string()
        } else {
            extension.to_string()
        };
        self.file_extension = ext;
        self
    }

    /// Set a suggested filename for the download
    pub fn with_filename(mut self, filename: String) -> Self {
        self.suggested_filename = Some(filename);
        self
    }

    /// Save the content to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), CallerError> {
        std::fs::write(path, &self.content)?;
        Ok(())
    }

    /// Save to a file with automatic filename based on URL or content type
    pub fn save<P: AsRef<Path>>(&self, directory: P, base_name: &str) -> Result<String, CallerError> {
        let dir = directory.as_ref();
        std::fs::create_dir_all(dir)?;
        
        let filename = if let Some(suggested) = &self.suggested_filename {
            suggested.clone()
        } else {
            format!("{}.{}", base_name, self.file_extension)
        };
        
        let file_path = dir.join(&filename);
        self.save_to_file(&file_path)?;
        
        Ok(filename)
    }

    /// Get the file size in bytes
    pub fn size(&self) -> usize {
        self.content.len()
    }

    /// Get human-readable file size
    pub fn size_human(&self) -> String {
        let bytes = self.content.len() as f64;
        let units = ["B", "KB", "MB", "GB"];
        let mut size = bytes;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < units.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        format!("{:.2} {}", size, units[unit_index])
    }

    /// Try to parse content as UTF-8 text
    pub fn as_text(&self) -> Result<&str, CallerError> {
        std::str::from_utf8(&self.content)
            .map_err(|e| CallerError::IoError(format!("Failed to decode as UTF-8: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_extension_from_mime() {
        assert_eq!(DownloadResult::detect_extension_from_mime("image/jpeg"), "jpg");
        assert_eq!(DownloadResult::detect_extension_from_mime("image/png"), "png");
        assert_eq!(DownloadResult::detect_extension_from_mime("application/json"), "json");
        assert_eq!(DownloadResult::detect_extension_from_mime("text/html"), "html");
        assert_eq!(DownloadResult::detect_extension_from_mime("application/pdf"), "pdf");
        assert_eq!(DownloadResult::detect_extension_from_mime("application/zip"), "zip");
        assert_eq!(DownloadResult::detect_extension_from_mime("application/octet-stream"), "bin");
    }

    #[test]
    fn test_detect_extension_with_charset() {
        assert_eq!(
            DownloadResult::detect_extension_from_mime("text/html; charset=utf-8"),
            "html"
        );
        assert_eq!(
            DownloadResult::detect_extension_from_mime("application/json; charset=utf-8"),
            "json"
        );
    }
}