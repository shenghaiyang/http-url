use alloc::string::String;
use thiserror::Error;

/// Convenience alias for crate results using [`HttpUrlError`].
pub type Result<T> = core::result::Result<T, HttpUrlError>;

/// Errors that can occur when constructing or building an [`HttpUrl`][crate::HttpUrl].
///
/// URL parsing itself is performed by the `url` crate; this enum only covers
/// the thin layer on top: forwarding [`url::ParseError`] and enforcing the
/// "http/https only" invariant.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum HttpUrlError {
    /// A URL string could not be parsed by the `url` crate.
    #[error("URL parse error: {0}")]
    Parse(#[from] url::ParseError),

    /// The URL's scheme is not `http` or `https`.
    #[error("unsupported scheme: '{0}' (only 'http' and 'https' are allowed)")]
    UnsupportedScheme(String),

    /// Builder validation failed (e.g. a required component was missing).
    #[error("builder validation error: {0}")]
    BuilderValidation(String),
}
