use crate::shared::error::CallerError;
use reqwest::StatusCode;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Download result containing file information and content
#[derive(Debug, Clone)]
pub struct DownloadResult {
    /// HTTP response status.
    pub status_code: StatusCode,
    /// Complete buffered response bytes.
    pub content: Vec<u8>,
    /// Response content type, defaulting to `application/octet-stream`.
    pub content_type: String,
    /// File extension inferred from the content type or set by the caller.
    pub file_extension: String,
    /// Safe filename candidate extracted from response metadata or set explicitly.
    pub suggested_filename: Option<String>,
}

impl DownloadResult {
    /// Create a new download result from HTTP response
    pub fn from_response(
        status_code: StatusCode,
        content: Vec<u8>,
        content_type: Option<String>,
    ) -> DownloadResult {
        let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_string());

        // Auto-detect file extension from content type
        let file_extension = Self::detect_extension_from_mime(&content_type);

        DownloadResult {
            status_code,
            content,
            content_type,
            file_extension,
            suggested_filename: None,
        }
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
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
                "docx".to_string()
            }
            "application/vnd.ms-excel" => "xls".to_string(),
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => {
                "xlsx".to_string()
            }
            "application/vnd.ms-powerpoint" => "ppt".to_string(),
            "application/vnd.openxmlformats-officedocument.presentationml.presentation" => {
                "pptx".to_string()
            }
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
        let ext = if let Some(stripped) = extension.strip_prefix('.') {
            stripped.to_string()
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
    pub fn save<P: AsRef<Path>>(
        &self,
        directory: P,
        base_name: &str,
    ) -> Result<String, CallerError> {
        let dir = directory.as_ref();
        std::fs::create_dir_all(dir)?;

        let filename = if let Some(suggested) = &self.suggested_filename {
            suggested.clone()
        } else {
            format!("{}.{}", base_name, self.file_extension)
        };

        validate_filename(&filename)?;

        let file_path = dir.join(&filename);
        ensure_path_is_within_directory(dir, &file_path, &filename)?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file_path)
            .map_err(|error| CallerError::io(format!("creating download '{filename}'"), error))?;
        if let Err(error) = file.write_all(&self.content) {
            drop(file);
            let _ = std::fs::remove_file(&file_path);
            return Err(CallerError::io(
                format!("writing download '{filename}'"),
                error,
            ));
        }

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
            .map_err(|e| CallerError::text_decoding_error(e.to_string()))
    }
}

fn validate_filename(filename: &str) -> Result<(), CallerError> {
    let invalid = filename.is_empty()
        || filename == "."
        || filename == ".."
        || filename.contains('/')
        || filename.contains('\\')
        || filename.contains('\0')
        || filename.chars().any(char::is_control)
        || Path::new(filename).is_absolute();

    if invalid {
        return Err(CallerError::UnsafeDownloadFilename {
            filename: filename.to_string(),
        });
    }

    Ok(())
}

fn ensure_path_is_within_directory(
    directory: &Path,
    file_path: &Path,
    filename: &str,
) -> Result<(), CallerError> {
    let expected: PathBuf = directory.join(filename);
    if file_path != expected || file_path.parent() != Some(directory) {
        return Err(CallerError::UnsafeDownloadFilename {
            filename: filename.to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_extension_from_mime() {
        assert_eq!(
            DownloadResult::detect_extension_from_mime("image/jpeg"),
            "jpg"
        );
        assert_eq!(
            DownloadResult::detect_extension_from_mime("image/png"),
            "png"
        );
        assert_eq!(
            DownloadResult::detect_extension_from_mime("application/json"),
            "json"
        );
        assert_eq!(
            DownloadResult::detect_extension_from_mime("text/html"),
            "html"
        );
        assert_eq!(
            DownloadResult::detect_extension_from_mime("application/pdf"),
            "pdf"
        );
        assert_eq!(
            DownloadResult::detect_extension_from_mime("application/zip"),
            "zip"
        );
        assert_eq!(
            DownloadResult::detect_extension_from_mime("application/octet-stream"),
            "bin"
        );
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

    #[test]
    fn rejects_unsafe_suggested_filenames() {
        let directory =
            std::env::temp_dir().join(format!("caller-download-test-{}", std::process::id()));
        let result = DownloadResult::from_response(
            StatusCode::OK,
            b"content".to_vec(),
            Some("text/plain".to_string()),
        )
        .with_filename("../outside.txt".to_string());

        let error = result.save(&directory, "fallback").unwrap_err();
        assert!(matches!(error, CallerError::UnsafeDownloadFilename { .. }));
        assert!(!directory.join("../outside.txt").exists());
        std::fs::remove_dir(directory).ok();
    }

    #[test]
    fn automatic_save_does_not_overwrite_existing_file() {
        let directory = std::env::temp_dir().join(format!(
            "caller-download-existing-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("file.txt");
        std::fs::write(&path, b"existing").unwrap();
        let result = DownloadResult::from_response(
            StatusCode::OK,
            b"replacement".to_vec(),
            Some("text/plain".to_string()),
        );

        assert!(result.save(&directory, "file").is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"existing");

        std::fs::remove_file(path).ok();
        std::fs::remove_dir(directory).ok();
    }
}
