use alloc::format;
use alloc::string::String;

use core::convert::TryFrom;
use core::fmt;
use core::str::FromStr;

use crate::builder::HttpUrlBuilder;
use crate::error::{HttpUrlError, Result};
use crate::scheme::Scheme;

/// An immutable HTTP or HTTPS URL.
///
/// `HttpUrl` is a thin newtype around [`url::Url`] that enforces the
/// "scheme is `http` or `https`" invariant at construction time. Parsing,
/// percent-encoding, normalization and all component access are provided by
/// the `url` crate — reach them through [`HttpUrl::as_url`] or
/// [`HttpUrl::into_url`]. This crate focuses on the [`HttpUrlBuilder`] API
/// for constructing URLs programmatically, plus a typed [`Scheme`] accessor
/// that reflects the http/https invariant.
///
/// # Example
///
/// ```
/// use http_url::HttpUrl;
///
/// let url = HttpUrl::parse("https://example.com/path?a=1#frag").unwrap();
/// assert_eq!(url.scheme(), "https");
/// assert_eq!(url.as_url().host_str(), Some("example.com"));
/// assert_eq!(url.as_url().path(), "/path");
/// assert_eq!(url.as_url().query(), Some("a=1"));
/// assert_eq!(url.as_url().fragment(), Some("frag"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HttpUrl(url::Url);

impl HttpUrl {
    // ── construction ───────────────────────────────────────────────

    /// Parse a URL string into an `HttpUrl`.
    ///
    /// The URL must have an `http://` or `https://` scheme; all other parsing
    /// behavior follows the WHATWG URL Standard as implemented by the `url`
    /// crate.
    pub fn parse(url: &str) -> Result<Self> {
        Self::from_url(url::Url::parse(url)?)
    }

    /// Wrap an existing [`url::Url`], returning an error if its scheme is not
    /// `http` or `https`.
    pub fn from_url(url: url::Url) -> Result<Self> {
        // `Scheme::from_str` validates the scheme; the `Scheme` value itself
        // is not needed here since it is recoverable from `url.scheme()`.
        let _ = Scheme::from_str(url.scheme())?;
        Ok(Self(url))
    }

    /// Borrow the underlying [`url::Url`], through which all URL components
    /// (host, path, query, fragment, …) are accessible.
    pub fn as_url(&self) -> &url::Url {
        &self.0
    }

    /// Consume this `HttpUrl` and return the underlying [`url::Url`].
    pub fn into_url(self) -> url::Url {
        self.0
    }

    // ── builder ────────────────────────────────────────────────────

    /// Create a new [`HttpUrlBuilder`].
    pub fn builder() -> HttpUrlBuilder {
        HttpUrlBuilder::new()
    }

    /// Create a builder pre-populated with this URL's components, so it can be
    /// modified and rebuilt.
    ///
    /// # Example
    ///
    /// ```
    /// use http_url::HttpUrl;
    ///
    /// let url = HttpUrl::parse("https://example.com/a/b?q=1").unwrap();
    /// let url2 = url.new_builder()
    ///     .add_path_segment("c")
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(url2.as_url().path(), "/a/b/c");
    /// ```
    pub fn new_builder(&self) -> HttpUrlBuilder {
        // The scheme invariant guarantees `from_url` cannot fail here.
        HttpUrlBuilder::from_url(self.0.clone())
            .expect("invariant: HttpUrl scheme is always http or https")
    }

    // ── typed accessor ─────────────────────────────────────────────

    /// Returns the URL scheme as a typed [`Scheme`].
    ///
    /// This is the typed mirror of [`url::Url::scheme`]; for a valid
    /// `HttpUrl` it is always `http` or `https`.
    pub fn scheme(&self) -> Scheme {
        // The invariant holds by construction, so `from_str` cannot fail.
        Scheme::from_str(self.0.scheme())
            .expect("invariant: HttpUrl scheme is always http or https")
    }

    // ── convenience helpers ────────────────────────────────────────

    /// Compute the top private domain (eTLD+1) of this URL's host.
    ///
    /// Returns `None` if the host is an IP address or does not have enough
    /// domain labels. This is a simplified implementation that assumes the
    /// last two labels form the private domain; a full implementation would
    /// consult the Public Suffix List.
    pub fn top_private_domain(&self) -> Option<String> {
        let host = self.0.host_str()?;
        // Reject IP-address hosts (no alphabetic characters).
        if !host.bytes().any(|b| b.is_ascii_alphabetic()) {
            return None;
        }
        // Take the last two dot-separated labels.
        let (rest, last) = host.rsplit_once('.')?;
        let (_, second_last) = rest.rsplit_once('.').unwrap_or(("", rest));
        Some(format!("{}.{}", second_last, last))
    }
}

impl fmt::Display for HttpUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl FromStr for HttpUrl {
    type Err = HttpUrlError;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

// ── interop with url::Url ─────────────────────────────────────────

impl From<HttpUrl> for url::Url {
    fn from(http: HttpUrl) -> Self {
        http.into_url()
    }
}

impl TryFrom<url::Url> for HttpUrl {
    type Error = HttpUrlError;

    fn try_from(url: url::Url) -> Result<Self> {
        Self::from_url(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn parse_simple() {
        let url = HttpUrl::parse("http://example.com").unwrap();
        assert_eq!(url.scheme(), Scheme::Http);
        assert_eq!(url.as_url().host_str(), Some("example.com"));
        assert_eq!(url.as_url().port(), None); // default port not stored
        assert_eq!(url.as_url().path(), "/");
    }

    #[test]
    fn parse_https() {
        let url = HttpUrl::parse("https://example.com/path").unwrap();
        assert_eq!(url.scheme(), Scheme::Https);
        assert_eq!(url.as_url().port(), None);
        assert_eq!(url.as_url().path(), "/path");
    }

    #[test]
    fn parse_explicit_port() {
        let url = HttpUrl::parse("http://example.com:8080/").unwrap();
        assert_eq!(url.as_url().port(), Some(8080));
    }

    #[test]
    fn parse_default_port_omitted() {
        // The `url` crate normalizes the default port away.
        let url = HttpUrl::parse("http://example.com:80/").unwrap();
        assert_eq!(url.as_url().port(), None);
    }

    #[test]
    fn parse_userinfo() {
        let url = HttpUrl::parse("http://user:pass@example.com/").unwrap();
        assert_eq!(url.as_url().username(), "user");
        assert_eq!(url.as_url().password(), Some("pass"));
    }

    #[test]
    fn parse_username_only() {
        let url = HttpUrl::parse("http://user@example.com/").unwrap();
        assert_eq!(url.as_url().username(), "user");
        assert_eq!(url.as_url().password(), None);
    }

    #[test]
    fn parse_query() {
        let url = HttpUrl::parse("http://example.com/?a=1&b=2").unwrap();
        let pairs: Vec<(String, String)> = url
            .as_url()
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        assert_eq!(
            pairs,
            vec![
                ("a".to_string(), "1".to_string()),
                ("b".to_string(), "2".to_string())
            ]
        );
    }

    #[test]
    fn parse_fragment() {
        let url = HttpUrl::parse("http://example.com/#section").unwrap();
        assert_eq!(url.as_url().fragment(), Some("section"));
    }

    #[test]
    fn parse_encoded_path() {
        let url = HttpUrl::parse("http://example.com/hello%20world").unwrap();
        assert_eq!(url.as_url().path(), "/hello%20world");
    }

    #[test]
    fn parse_ipv6() {
        let url = HttpUrl::parse("http://[::1]:8080/path").unwrap();
        assert_eq!(url.as_url().host_str(), Some("[::1]"));
        assert_eq!(url.as_url().port(), Some(8080));
    }

    #[test]
    fn parse_rejects_empty() {
        assert!(HttpUrl::parse("").is_err());
    }

    #[test]
    fn parse_rejects_other_schemes() {
        assert!(HttpUrl::parse("ftp://example.com").is_err());
        assert!(HttpUrl::parse("file:///etc/passwd").is_err());
    }

    #[test]
    fn parse_rejects_missing_host() {
        // `http://` with no host is invalid per the URL Standard.
        assert!(HttpUrl::parse("http://").is_err());
    }

    #[test]
    fn from_str_roundtrip() {
        let inputs = [
            "http://example.com/",
            "https://example.com/path/to?q=1&r=2#frag",
            "http://user:pass@host.com:8080/a/b/c",
            "http://[::1]:9090/path",
            "http://example.com/hello%20world",
        ];
        for input in &inputs {
            let url: HttpUrl = input.parse().unwrap();
            assert_eq!(url.to_string(), *input);
        }
    }

    #[test]
    fn display_matches_url_string() {
        let url = HttpUrl::parse("https://example.com/a?b=2#c").unwrap();
        assert_eq!(url.to_string(), url.as_url().as_str());
    }

    // ── relative resolution (via url::Url::join) ─────────────────

    #[test]
    fn resolve_absolute() {
        let base = HttpUrl::parse("http://example.com/a/b").unwrap();
        let resolved =
            HttpUrl::from_url(base.as_url().join("http://other.com/c").unwrap()).unwrap();
        assert_eq!(resolved.as_url().host_str(), Some("other.com"));
        assert_eq!(resolved.as_url().path(), "/c");
    }

    #[test]
    fn resolve_relative() {
        let base = HttpUrl::parse("http://example.com/a/b").unwrap();
        let resolved = HttpUrl::from_url(base.as_url().join("c").unwrap()).unwrap();
        assert_eq!(resolved.as_url().path(), "/a/c");
    }

    #[test]
    fn resolve_absolute_path() {
        let base = HttpUrl::parse("http://example.com/a/b").unwrap();
        let resolved = HttpUrl::from_url(base.as_url().join("/c/d").unwrap()).unwrap();
        assert_eq!(resolved.as_url().path(), "/c/d");
    }

    #[test]
    fn resolve_protocol_relative() {
        let base = HttpUrl::parse("http://example.com/a/b").unwrap();
        let resolved = HttpUrl::from_url(base.as_url().join("//other.com/c").unwrap()).unwrap();
        assert_eq!(resolved.scheme(), Scheme::Http);
        assert_eq!(resolved.as_url().host_str(), Some("other.com"));
        assert_eq!(resolved.as_url().path(), "/c");
    }

    #[test]
    fn resolve_query_only() {
        let base = HttpUrl::parse("http://example.com/a/b?old=1").unwrap();
        let resolved = HttpUrl::from_url(base.as_url().join("?new=2").unwrap()).unwrap();
        assert_eq!(
            resolved
                .as_url()
                .query_pairs()
                .next()
                .map(|(k, v)| (k.into_owned(), v.into_owned())),
            Some(("new".to_string(), "2".to_string()))
        );
    }

    #[test]
    fn resolve_fragment_only() {
        let base = HttpUrl::parse("http://example.com/a/b#old").unwrap();
        let resolved = HttpUrl::from_url(base.as_url().join("#new").unwrap()).unwrap();
        assert_eq!(resolved.as_url().fragment(), Some("new"));
    }

    // ── domain helpers ─────────────────────────────────────────────

    #[test]
    fn top_private_domain() {
        let url = HttpUrl::parse("http://www.example.com/path").unwrap();
        assert_eq!(url.top_private_domain(), Some("example.com".to_string()));
    }

    #[test]
    fn top_private_domain_ip_is_none() {
        let url = HttpUrl::parse("http://192.168.1.1/").unwrap();
        assert_eq!(url.top_private_domain(), None);
    }

    // ── interop with url::Url ──────────────────────────────────────

    #[test]
    fn from_url_rejects_non_http() {
        let u = url::Url::parse("ftp://example.com").unwrap();
        assert!(HttpUrl::try_from(u).is_err());
    }

    #[test]
    fn url_roundtrip() {
        let u = url::Url::parse("https://example.com/path?q=1").unwrap();
        let http = HttpUrl::try_from(u.clone()).unwrap();
        assert_eq!(http.as_url(), &u);
        let back: url::Url = http.into();
        assert_eq!(back, u);
    }

    #[test]
    fn new_builder_rebuilds_equal() {
        let url = HttpUrl::parse("http://example.com/path?q=1#frag").unwrap();
        let url2 = url.new_builder().build().unwrap();
        assert_eq!(url, url2);
    }
}
