use thiserror::Error;

#[derive(Error, Debug)]
pub enum SummarizerError {
    #[error("Failed to read directory: {path}")]
    DirectoryRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to read file: {path}")]
    FileRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write file: {path}")]
    FileWrite {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("code-index not available: {message}")]
    CodeIndexUnavailable { message: String },

    #[error("code-index query failed: {message}")]
    CodeIndexQuery { message: String },

    #[error("Git repository not found at: {path}")]
    GitNotFound { path: String },

    #[error("Git operation failed: {message}")]
    GitOperation { message: String },

    #[error("Template rendering failed: {template}")]
    TemplateRender {
        template: String,
        #[source]
        source: tera::Error,
    },

    #[error("Template not found: {name}")]
    TemplateNotFound { name: String },

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("LLM request failed: {message}")]
    LlmRequest { message: String },

    #[error("Invalid path: {path}")]
    InvalidPath { path: String },

    #[error("Project analysis failed: {message}")]
    AnalysisFailed { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_directory_read() {
        let err = SummarizerError::DirectoryRead {
            path: "/some/path".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        };
        assert!(err.to_string().contains("/some/path"));
        assert!(err.to_string().contains("directory"));
    }

    #[test]
    fn test_error_display_code_index_unavailable() {
        let err = SummarizerError::CodeIndexUnavailable {
            message: "daemon not running".to_string(),
        };
        assert!(err.to_string().contains("code-index"));
        assert!(err.to_string().contains("daemon not running"));
    }

    #[test]
    fn test_error_display_template_not_found() {
        let err = SummarizerError::TemplateNotFound {
            name: "architecture".to_string(),
        };
        assert!(err.to_string().contains("architecture"));
    }
}
