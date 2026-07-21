use core::str::FromStr;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use url::Url;

use crate::HttpUrl;
use crate::error::{HttpUrlError, Result};
use crate::scheme::Scheme;
use crate::util::percent_decode;

/// Builder for constructing HTTP/HTTPS URLs programmatically.
///
/// `HttpUrlBuilder` is the focus of this crate. It accumulates URL components
/// as **decoded** Rust strings and, on [`build`][Self::build] / [`build_url`][Self::build_url],
/// hands them to the `url` crate, which performs all percent-encoding,
/// normalization and validation according to the WHATWG URL Standard.
///
/// # Example
///
/// ```
/// use http_url::{HttpUrl, Scheme};
///
/// let url = HttpUrl::builder()
///     .scheme(Scheme::Https)
///     .host("example.com")
///     .add_path_segment("api")
///     .add_path_segment("v1")
///     .add_query_parameter("key", "value")
///     .fragment("section")
///     .build()
///     .unwrap();
///
/// assert_eq!(url.to_string(), "https://example.com/api/v1?key=value#section");
/// ```
#[derive(Debug, Clone)]
pub struct HttpUrlBuilder {
    /// The URL scheme, or `None` if not yet set.
    scheme: Option<Scheme>,
    /// Username for basic authentication.
    username: String,
    /// Password for basic authentication.
    password: String,
    /// The host, or `None` if not yet set.
    host: Option<String>,
    /// The port, or `None` to use the scheme's default.
    port: Option<u16>,
    /// Decoded path segments.
    path_segments: Vec<String>,
    /// Decoded query parameter pairs.
    query_pairs: Vec<(String, String)>,
    /// The fragment (decoded), or `None`.
    fragment: Option<String>,
}

impl HttpUrlBuilder {
    /// Create a new empty builder.
    pub fn new() -> Self {
        Self {
            scheme: None,
            username: String::new(),
            password: String::new(),
            host: None,
            port: None,
            path_segments: Vec::new(),
            query_pairs: Vec::new(),
            fragment: None,
        }
    }

    /// Create a builder pre-populated from an existing [`url::Url`].
    ///
    /// The URL must have an `http` or `https` scheme. The returned builder is
    /// ready to be modified and rebuilt via [`build`][Self::build] or
    /// [`build_url`][Self::build_url].
    ///
    /// # Example
    ///
    /// ```
    /// use http_url::HttpUrlBuilder;
    ///
    /// let u = url::Url::parse("https://example.com/a/b?q=1").unwrap();
    /// let url = HttpUrlBuilder::from_url(u)
    ///     .unwrap()
    ///     .add_path_segment("c")
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(url.as_url().path(), "/a/b/c");
    /// ```
    pub fn from_url(url: Url) -> Result<Self> {
        let scheme = Scheme::from_str(url.scheme())?;
        Ok(Self {
            scheme: Some(scheme),
            username: url.username().to_string(),
            password: url.password().unwrap_or("").to_string(),
            host: Some(url.host_str().unwrap_or("").to_string()),
            port: url.port(),
            path_segments: url
                .path_segments()
                .map(|segs| segs.map(percent_decode).collect::<Vec<_>>())
                .unwrap_or_default(),
            query_pairs: url
                .query_pairs()
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect(),
            fragment: url.fragment().map(percent_decode),
        })
    }

    /// Create a builder pre-populated by parsing a URL string.
    ///
    /// Equivalent to [`HttpUrlBuilder::from_url`] applied to the result of
    /// `url::Url::parse`.
    pub fn parse(url: &str) -> Result<Self> {
        Self::from_url(Url::parse(url)?)
    }

    /// Set the scheme.
    pub fn scheme(mut self, scheme: Scheme) -> Self {
        self.scheme = Some(scheme);
        self
    }

    /// Set the username for basic auth.
    pub fn username(mut self, username: &str) -> Self {
        self.username = username.to_string();
        self
    }

    /// Set the password for basic auth.
    pub fn password(mut self, password: &str) -> Self {
        self.password = password.to_string();
        self
    }

    /// Set the host (domain name or IP address literal, including bracketed
    /// IPv6 such as `[::1]`).
    pub fn host(mut self, host: &str) -> Self {
        self.host = Some(host.to_string());
        self
    }

    /// Set the port. Use [`remove_port`][Self::remove_port] to revert to the
    /// scheme's default.
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Revert to the scheme's default port.
    pub fn remove_port(mut self) -> Self {
        self.port = None;
        self
    }

    /// Add a single path segment (decoded form; it is percent-encoded on
    /// build).
    pub fn add_path_segment(mut self, segment: &str) -> Self {
        self.path_segments.push(segment.to_string());
        self
    }

    /// Add multiple path segments at once (decoded form).
    pub fn add_path_segments(mut self, segments: &[&str]) -> Self {
        for seg in segments {
            self.path_segments.push(seg.to_string());
        }
        self
    }

    /// Replace the entire path with the segments of `path` (decoded form).
    ///
    /// `path` is split on `/`; a leading `/` is ignored and empty segments are
    /// dropped. Each segment is stored as-is (decoded) and re-encoded on
    /// build. No percent-decoding is applied to the input — pass a decoded
    /// path.
    pub fn set_path(mut self, path: &str) -> Self {
        self.path_segments.clear();
        for seg in path.trim_start_matches('/').split('/') {
            if !seg.is_empty() {
                self.path_segments.push(seg.to_string());
            }
        }
        self
    }

    /// Add a query parameter (name and value in decoded form). Both are
    /// percent-encoded on build using `application/x-www-form-urlencoded`
    /// semantics (space becomes `+`).
    pub fn add_query_parameter(mut self, name: &str, value: &str) -> Self {
        self.query_pairs.push((name.to_string(), value.to_string()));
        self
    }

    /// Remove all query parameters with the given name.
    pub fn remove_query_parameter(mut self, name: &str) -> Self {
        self.query_pairs.retain(|(n, _)| n != name);
        self
    }

    /// Clear all query parameters.
    pub fn clear_query_parameters(mut self) -> Self {
        self.query_pairs.clear();
        self
    }

    /// Set the fragment (decoded form).
    pub fn fragment(mut self, fragment: &str) -> Self {
        self.fragment = Some(fragment.to_string());
        self
    }

    /// Remove the fragment.
    pub fn remove_fragment(mut self) -> Self {
        self.fragment = None;
        self
    }

    /// Consume the builder and produce a [`url::Url`].
    ///
    /// The scheme is always `http` or `https` (it is a typed [`Scheme`] field),
    /// so the result is always a valid HTTP(S) URL. Returns an error only if
    /// required components (scheme, host) are missing or the `url` crate
    /// rejects the assembled URL (e.g. an invalid host).
    ///
    /// Use [`build`][Self::build] instead if you want an [`HttpUrl`] wrapper.
    pub fn build_url(self) -> Result<Url> {
        let scheme = self
            .scheme
            .ok_or_else(|| HttpUrlError::BuilderValidation("scheme is required".to_string()))?;
        let host = self
            .host
            .ok_or_else(|| HttpUrlError::BuilderValidation("host is required".to_string()))?;

        // Assemble the base URL; `url::Url::parse` performs host validation,
        // IDNA/punycode normalization, etc.
        let mut url = Url::parse(&format!("{}://{}", scheme.as_str(), host))?;

        // Authority additions.
        if !self.username.is_empty() {
            let _ = url.set_username(&self.username);
        }
        if !self.password.is_empty() {
            let _ = url.set_password(Some(&self.password));
        }
        if let Some(port) = self.port {
            let _ = url.set_port(Some(port));
        }

        // Path segments. Only overwrite when the builder carries segments, so
        // a builder derived via `from_url` keeps the original path when the
        // caller never touches it.
        if !self.path_segments.is_empty() {
            let mut segments = url.path_segments_mut().map_err(|_| {
                HttpUrlError::BuilderValidation("URL cannot have a path".to_string())
            })?;
            segments.clear();
            for seg in &self.path_segments {
                segments.push(seg);
            }
        }

        // Query parameters. Explicitly clear when empty so that a builder
        // derived via `from_url` does not retain a stale query after the
        // caller calls `clear_query_parameters`.
        if !self.query_pairs.is_empty() {
            let mut query = url.query_pairs_mut();
            for (name, value) in &self.query_pairs {
                query.append_pair(name, value);
            }
        } else {
            url.set_query(None);
        }

        url.set_fragment(self.fragment.as_deref());

        Ok(url)
    }

    /// Consume the builder and produce an [`HttpUrl`] — a newtype over
    /// [`url::Url`] enforcing the http/https invariant.
    ///
    /// Equivalent to [`HttpUrl::from_url`] applied to [`build_url`][Self::build_url].
    pub fn build(self) -> Result<HttpUrl> {
        HttpUrl::from_url(self.build_url()?)
    }
}

impl Default for HttpUrlBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FromStr for HttpUrlBuilder {
    type Err = HttpUrlError;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_basic() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Https)
            .host("example.com")
            .build()
            .unwrap();
        assert_eq!(url.to_string(), "https://example.com/");
    }

    #[test]
    fn builder_full() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Https)
            .username("user")
            .password("pass")
            .host("example.com")
            .port(8080)
            .add_path_segment("api")
            .add_path_segment("v1")
            .add_query_parameter("key", "value")
            .fragment("section")
            .build()
            .unwrap();
        assert_eq!(
            url.to_string(),
            "https://user:pass@example.com:8080/api/v1?key=value#section"
        );
    }

    #[test]
    fn builder_path_segments() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Http)
            .host("example.com")
            .add_path_segment("a")
            .add_path_segment("b")
            .add_path_segment("c")
            .build()
            .unwrap();
        assert_eq!(url.as_url().path(), "/a/b/c");
    }

    #[test]
    fn builder_path_encoding() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Http)
            .host("example.com")
            .add_path_segment("hello world")
            .build()
            .unwrap();
        assert_eq!(url.as_url().path(), "/hello%20world");
    }

    #[test]
    fn builder_unicode_segment() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Http)
            .host("example.com")
            .add_path_segment("中文")
            .build()
            .unwrap();
        assert_eq!(url.as_url().path(), "/%E4%B8%AD%E6%96%87");
    }

    #[test]
    fn builder_set_path() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Http)
            .host("example.com")
            .set_path("/a/b/c")
            .build()
            .unwrap();
        assert_eq!(url.as_url().path(), "/a/b/c");
    }

    #[test]
    fn builder_add_path_segments() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Http)
            .host("example.com")
            .add_path_segments(&["a", "b", "c"])
            .build()
            .unwrap();
        assert_eq!(url.as_url().path(), "/a/b/c");
    }

    #[test]
    fn builder_query_encoding() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Http)
            .host("example.com")
            .add_query_parameter("name", "hello world")
            .build()
            .unwrap();
        // form_urlencoded uses `+` for space.
        assert_eq!(url.as_url().query(), Some("name=hello+world"));
    }

    #[test]
    fn builder_clear_query_parameters() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Http)
            .host("example.com")
            .add_query_parameter("a", "1")
            .clear_query_parameters()
            .build()
            .unwrap();
        assert_eq!(url.as_url().query(), None);
    }

    #[test]
    fn builder_remove_query_parameter() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Https)
            .host("example.com")
            .add_query_parameter("a", "1")
            .add_query_parameter("b", "2")
            .remove_query_parameter("a")
            .build()
            .unwrap();
        assert_eq!(url.as_url().query_pairs().find(|(k, _)| k == "a"), None);
        assert_eq!(
            url.as_url()
                .query_pairs()
                .find(|(k, _)| k == "b")
                .map(|(_, v)| v.into_owned()),
            Some("2".to_string())
        );
    }

    #[test]
    fn builder_remove_fragment() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Https)
            .host("example.com")
            .fragment("sec")
            .remove_fragment()
            .build()
            .unwrap();
        assert_eq!(url.as_url().fragment(), None);
    }

    #[test]
    fn builder_remove_port() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Http)
            .host("example.com")
            .port(8080)
            .remove_port()
            .build()
            .unwrap();
        assert_eq!(url.as_url().port(), None); // default port normalized away
    }

    #[test]
    fn builder_missing_scheme() {
        assert!(HttpUrl::builder().host("example.com").build().is_err());
    }

    #[test]
    fn builder_missing_host() {
        assert!(HttpUrl::builder().scheme(Scheme::Http).build().is_err());
    }

    #[test]
    fn builder_ipv6_host() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Http)
            .host("[::1]")
            .port(8080)
            .build()
            .unwrap();
        assert_eq!(url.as_url().host_str(), Some("[::1]"));
        assert_eq!(url.as_url().port(), Some(8080));
    }

    #[test]
    fn builder_idn_normalization() {
        // The url crate normalizes internationalized domain names.
        let url = HttpUrl::builder()
            .scheme(Scheme::Http)
            .host("Bücher.de")
            .build()
            .unwrap();
        assert_eq!(url.as_url().host_str(), Some("xn--bcher-kva.de"));
    }

    #[test]
    fn builder_new_builder_roundtrip() {
        let url = HttpUrl::parse("https://example.com/a/b?x=1&y=2#frag").unwrap();
        let url2 = url.new_builder().build().unwrap();
        assert_eq!(url, url2);
    }

    #[test]
    fn builder_new_builder_modify() {
        let url = HttpUrl::parse("https://example.com/a/b?x=1").unwrap();
        let url2 = url
            .new_builder()
            .add_path_segment("c")
            .add_query_parameter("y", "2")
            .build()
            .unwrap();
        assert_eq!(url2.as_url().path(), "/a/b/c");
        assert_eq!(
            url2.as_url()
                .query_pairs()
                .find(|(k, _)| k == "x")
                .map(|(_, v)| v.into_owned()),
            Some("1".to_string())
        );
        assert_eq!(
            url2.as_url()
                .query_pairs()
                .find(|(k, _)| k == "y")
                .map(|(_, v)| v.into_owned()),
            Some("2".to_string())
        );
    }

    #[test]
    fn builder_from_url() {
        let u = url::Url::parse("https://example.com/a/b?q=1#frag").unwrap();
        let url = HttpUrlBuilder::from_url(u.clone())
            .unwrap()
            .add_path_segment("c")
            .build()
            .unwrap();
        assert_eq!(url.as_url().path(), "/a/b/c");
        assert_eq!(
            url.as_url()
                .query_pairs()
                .find(|(k, _)| k == "q")
                .map(|(_, v)| v.into_owned()),
            Some("1".to_string())
        );
        assert_eq!(url.as_url().fragment(), Some("frag"));
    }

    #[test]
    fn builder_from_url_rejects_non_http() {
        let u = url::Url::parse("ftp://example.com").unwrap();
        assert!(HttpUrlBuilder::from_url(u).is_err());
    }

    #[test]
    fn builder_parse() {
        let b: HttpUrlBuilder = "https://example.com/a/b".parse().unwrap();
        let url = b.add_path_segment("c").build().unwrap();
        assert_eq!(url.as_url().path(), "/a/b/c");
    }

    #[test]
    fn builder_build_url() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Https)
            .host("example.com")
            .add_path_segment("api")
            .add_query_parameter("k", "v")
            .fragment("sec")
            .build_url()
            .unwrap();
        assert_eq!(url.as_str(), "https://example.com/api?k=v#sec");
    }

    #[test]
    fn builder_build_url_missing_scheme() {
        assert!(HttpUrl::builder().host("example.com").build_url().is_err());
    }

    #[test]
    fn builder_parse_rejects_non_http() {
        assert!(HttpUrlBuilder::parse("ftp://example.com").is_err());
    }
}
