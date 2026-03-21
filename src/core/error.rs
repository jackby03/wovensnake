use thiserror::Error;

#[derive(Error, Debug)]
pub enum WovenError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Zip extraction error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("Version parse error: {0}")]
    VersionParse(#[from] pep508_rs::pep440_rs::VersionParseError),

    #[error("Requirement parse error: {0}")]
    Pep508Parse(#[from] pep508_rs::Pep508Error),

    #[error("Template syntax error: {0}")]
    Template(#[from] indicatif::style::TemplateError),

    #[error("Python execution failed: {0}")]
    PythonExecution(String),

    #[error("Package resolution logic failed: {0}")]
    ResolutionConflict(String),

    #[error("Lockfile error: {0}")]
    Lockfile(String),

    #[error("Missing or corrupt package data: {0}")]
    CorruptPackage(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Async task join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("Generic execution failed: {0}")]
    Generic(String),
}

// Implement From<&str> and From<String> to generic WovenError for quick conversions
impl From<&str> for WovenError {
    fn from(s: &str) -> Self {
        Self::Generic(s.to_string())
    }
}

impl From<String> for WovenError {
    fn from(s: String) -> Self {
        Self::Generic(s)
    }
}
