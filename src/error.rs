use alloc::string::String;
use thiserror::Error;

/// Convenience alias for crate results using [`HttpUrlError`].
pub type Result<T> = core::result::Result<T, HttpUrlError>;

/// Errors that can occur when parsing or building an HttpUrl.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum HttpUrlError {
    /// The URL string is empty.
    #[error("URL is empty")]
    EmptyUrl,

    /// Missing scheme (e.g., "http", "https").
    #[error("missing scheme (expected 'http' or 'https')")]
    MissingScheme,

    /// Unsupported scheme (only "http" and "https" are supported).
    #[error("unsupported scheme: '{0}'")]
    UnsupportedScheme(String),

    /// Missing host after "://".
    #[error("missing host")]
    MissingHost,

    /// Invalid port number.
    #[error("invalid port: '{0}'")]
    InvalidPort(String),

    /// Port number out of range (must be 0–65535).
    #[error("port out of range: {0}")]
    PortOutOfRange(u64),

    /// Invalid host (e.g., empty label, too long, bad characters).
    #[error("invalid host: '{0}'")]
    InvalidHost(String),

    /// Invalid percent-encoding (e.g., "%ZZ" or "%XX" with non-hex chars).
    #[error("invalid percent-encoding: '{0}'")]
    InvalidPercentEncoding(String),

    /// Unicode normalization failure.
    #[error("unicode normalization error")]
    UnicodeError,

    /// Builder validation failed.
    #[error("builder validation error: {0}")]
    BuilderValidation(String),
}
