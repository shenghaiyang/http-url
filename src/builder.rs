use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::error::Result;
use crate::util::{host, percent};
use crate::{HttpUrl, HttpUrlError, Scheme};

/// Builder for constructing [`HttpUrl`] values programmatically.
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
    pub(crate) scheme: Option<Scheme>,
    /// Username for basic authentication.
    pub(crate) username: String,
    /// Password for basic authentication.
    pub(crate) password: String,
    /// The host, or `None` if not yet set.
    pub(crate) host: Option<String>,
    /// The port, or `None` to use the scheme's default.
    pub(crate) port: Option<u16>,
    /// Decoded path segments.
    pub(crate) path_segments: Vec<String>,
    /// Flattened query names and values (name0, value0, name1, value1, …).
    pub(crate) query_names_and_values: Vec<String>,
    /// The fragment (decoded), or `None`.
    pub(crate) fragment: Option<String>,
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
            query_names_and_values: Vec::new(),
            fragment: None,
        }
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

    /// Set the host (domain name or IP address).
    pub fn host(mut self, host: &str) -> Self {
        self.host = Some(host.to_string());
        self
    }

    /// Set the port. Use `None` to use the scheme's default.
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Add a path segment (decoded form, will be percent-encoded on build).
    pub fn add_path_segment(mut self, segment: &str) -> Self {
        self.path_segments.push(segment.to_string());
        self
    }

    /// Add multiple path segments.
    pub fn add_path_segments(mut self, segments: &[&str]) -> Self {
        for seg in segments {
            self.path_segments.push(seg.to_string());
        }
        self
    }

    /// Set the entire path from a decoded path string (e.g., `/a/b/c`).
    /// The path is split on `/` and each segment is decoded.
    pub fn set_path(mut self, path: &str) -> Self {
        self.path_segments.clear();
        let path = path.trim_start_matches('/');
        for seg in path.split('/') {
            if !seg.is_empty() {
                self.path_segments
                    .push(percent::decode(seg).unwrap_or_else(|_| seg.to_string()));
            }
        }
        self
    }

    /// Set the entire path from an already-encoded path string.
    /// Each segment is percent-decoded before storage.
    pub fn set_encoded_path(mut self, path: &str) -> Self {
        self.path_segments.clear();
        let path = path.trim_start_matches('/');
        for seg in path.split('/') {
            if !seg.is_empty() {
                let decoded = percent::decode(seg).unwrap_or_else(|_| seg.to_string());
                self.path_segments.push(decoded);
            }
        }
        self
    }

    /// Add a query parameter (name and value in decoded form).
    pub fn add_query_parameter(mut self, name: &str, value: &str) -> Self {
        self.query_names_and_values.push(name.to_string());
        self.query_names_and_values.push(value.to_string());
        self
    }

    /// Remove all query parameters with the given name.
    pub fn remove_query_parameter(mut self, name: &str) -> Self {
        let mut i = 0;
        while i < self.query_names_and_values.len() {
            if self.query_names_and_values[i] == name {
                self.query_names_and_values.remove(i);
                if i < self.query_names_and_values.len() {
                    self.query_names_and_values.remove(i);
                }
            } else {
                i += 2;
            }
        }
        self
    }

    /// Clear all query parameters.
    pub fn clear_query_parameters(mut self) -> Self {
        self.query_names_and_values.clear();
        self
    }

    /// Add a query parameter from already-encoded name and value.
    pub fn add_encoded_query_parameter(mut self, name: &str, value: &str) -> Self {
        let decoded_name = percent::decode(name).unwrap_or_else(|_| name.to_string());
        let decoded_value = percent::decode(value).unwrap_or_else(|_| value.to_string());
        self.query_names_and_values.push(decoded_name);
        self.query_names_and_values.push(decoded_value);
        self
    }

    /// Set the fragment (decoded form).
    pub fn fragment(mut self, fragment: &str) -> Self {
        self.fragment = Some(fragment.to_string());
        self
    }

    /// Set the fragment from an encoded form.
    pub fn encoded_fragment(mut self, fragment: &str) -> Self {
        self.fragment = percent::decode(fragment).ok();
        self
    }

    /// Remove the fragment.
    pub fn remove_fragment(mut self) -> Self {
        self.fragment = None;
        self
    }

    /// Consume the builder and produce an `HttpUrl`.
    ///
    /// Returns an error if required components (scheme, host) are missing.
    pub fn build(self) -> Result<HttpUrl> {
        let scheme = self
            .scheme
            .ok_or_else(|| HttpUrlError::BuilderValidation("scheme is required".to_string()))?;

        let host_str = self
            .host
            .ok_or_else(|| HttpUrlError::BuilderValidation("host is required".to_string()))?;

        let host = host::canonicalize_host(&host_str)?;

        let port = self.port.unwrap_or_else(|| scheme.default_port());

        Ok(HttpUrl {
            scheme,
            username: self.username,
            password: self.password,
            host,
            port,
            path_segments: self.path_segments,
            query_names_and_values: self.query_names_and_values,
            fragment: self.fragment,
        })
    }
}

impl Default for HttpUrlBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Https)
            .host("example.com")
            .build()
            .unwrap();
        assert_eq!(url.to_string(), "https://example.com/");
    }

    #[test]
    fn test_builder_full() {
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
    fn test_builder_path_segments() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Http)
            .host("example.com")
            .add_path_segment("a")
            .add_path_segment("b")
            .add_path_segment("c")
            .build()
            .unwrap();
        assert_eq!(url.path(), "/a/b/c");
        assert_eq!(url.encoded_path(), "/a/b/c");
    }

    #[test]
    fn test_builder_path_encoding() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Http)
            .host("example.com")
            .add_path_segment("hello world")
            .build()
            .unwrap();
        assert_eq!(url.encoded_path(), "/hello%20world");
        assert_eq!(url.path(), "/hello world");
    }

    #[test]
    fn test_builder_missing_scheme() {
        let result = HttpUrl::builder().host("example.com").build();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_missing_host() {
        let result = HttpUrl::builder().scheme(Scheme::Http).build();
        assert!(result.is_err());
    }

    #[test]
    fn test_unicode_in_builder() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Http)
            .host("example.com")
            .add_path_segment("\u{4e2d}\u{6587}")
            .build()
            .unwrap();
        assert_eq!(url.path(), "/\u{4e2d}\u{6587}");
    }

    #[test]
    fn test_builder_set_path() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Http)
            .host("example.com")
            .set_path("/a/b/c")
            .build()
            .unwrap();
        assert_eq!(url.path(), "/a/b/c");
    }

    #[test]
    fn test_builder_add_path_segments() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Http)
            .host("example.com")
            .add_path_segments(&["a", "b", "c"])
            .build()
            .unwrap();
        assert_eq!(url.path(), "/a/b/c");
    }

    #[test]
    fn test_builder_clear_query_parameters() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Http)
            .host("example.com")
            .add_query_parameter("a", "1")
            .clear_query_parameters()
            .build()
            .unwrap();
        assert_eq!(url.query(), None);
    }

    #[test]
    fn test_builder_remove_query_parameter() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Https)
            .host("example.com")
            .add_query_parameter("a", "1")
            .add_query_parameter("b", "2")
            .remove_query_parameter("a")
            .build()
            .unwrap();
        assert_eq!(url.query_parameter("a"), None);
        assert_eq!(url.query_parameter("b"), Some("2"));
    }
}
