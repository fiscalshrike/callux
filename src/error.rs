use std::fmt;

#[derive(Debug)]
pub enum CalendarError {
    AuthenticationFailed(String),
    ApiError(String),
    NetworkError(String),
    ConfigError(String),
    CacheError(String),
    ParseError(String),
}

impl fmt::Display for CalendarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalendarError::AuthenticationFailed(msg) => write!(f, "Authentication failed: {}", msg),
            CalendarError::ApiError(msg) => write!(f, "API error: {}", msg),
            CalendarError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            CalendarError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            CalendarError::CacheError(msg) => write!(f, "Cache error: {}", msg),
            CalendarError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for CalendarError {}

pub type Result<T> = std::result::Result<T, CalendarError>;