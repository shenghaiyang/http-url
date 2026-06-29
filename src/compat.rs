//! Conversions between [`HttpUrl`] and the [`url`] crate's [`Url`].
//!
//! This module is only available when the `url` feature is enabled.

use core::convert::TryFrom;

use url::Url;

use crate::HttpUrl;
use crate::error::Result;

// ── HttpUrl → url::Url ───────────────────────────────────────────

impl From<HttpUrl> for Url {
    /// Converts an `HttpUrl` into a `url::Url`.
    ///
    /// This conversion is infallible because `HttpUrl` always represents
    /// a syntactically valid HTTP or HTTPS URL.
    ///
    /// # Example
    ///
    /// ```
    /// use http_url::HttpUrl;
    /// let http = HttpUrl::parse("http://example.com/path?q=1").unwrap();
    /// let url: url::Url = http.into();
    /// assert_eq!(url.as_str(), "http://example.com/path?q=1");
    /// ```
    fn from(http: HttpUrl) -> Self {
        Url::parse(&http.to_url_string()).expect("HttpUrl should always produce a valid url::Url")
    }
}

impl From<&HttpUrl> for Url {
    fn from(http: &HttpUrl) -> Self {
        Url::parse(&http.to_url_string()).expect("HttpUrl should always produce a valid url::Url")
    }
}

// ── url::Url → HttpUrl (delegates to HttpUrl::parse) ─────────────

impl TryFrom<Url> for HttpUrl {
    type Error = crate::error::HttpUrlError;

    /// Tries to convert a `url::Url` into an `HttpUrl`.
    ///
    /// Delegates to [`HttpUrl::parse`] under the hood, so the same
    /// validation rules apply (only `http` / `https` schemes are
    /// accepted).
    ///
    /// # Example
    ///
    /// ```
    /// use core::convert::TryFrom;
    /// use http_url::HttpUrl;
    /// use url::Url;
    ///
    /// let u = Url::parse("https://user:pass@example.com:8080/path?a=1").unwrap();
    /// let http = HttpUrl::try_from(u).unwrap();
    /// assert_eq!(http.scheme(), "https");
    /// assert_eq!(http.host(), "example.com");
    /// assert_eq!(http.port(), 8080);
    /// ```
    fn try_from(url: Url) -> Result<Self> {
        HttpUrl::parse(url.as_str())
    }
}

impl TryFrom<&Url> for HttpUrl {
    type Error = crate::error::HttpUrlError;

    fn try_from(url: &Url) -> Result<Self> {
        HttpUrl::parse(url.as_str())
    }
}
