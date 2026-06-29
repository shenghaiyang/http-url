use core::fmt;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

#[cfg(test)]
use alloc::vec;

use crate::builder::HttpUrlBuilder;
use crate::error::{HttpUrlError, Result};
use crate::scheme::Scheme;
use crate::util::{host, parse, percent};
use core::fmt::Write;

/// An immutable, parsed HTTP or HTTPS URL.
///
/// # Example
///
/// ```
/// use http_url::HttpUrl;
///
/// let url = HttpUrl::parse("https://example.com/path?a=1#frag").unwrap();
/// assert_eq!(url.scheme(), "https");
/// assert_eq!(url.host(), "example.com");
/// assert_eq!(url.path(), "/path");
/// assert_eq!(url.query_parameter("a"), Some("1"));
/// assert_eq!(url.fragment(), Some("frag"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpUrl {
    /// The URL scheme (`http` or `https`).
    pub(crate) scheme: Scheme,
    /// Username for basic authentication (empty if none).
    pub(crate) username: String,
    /// Password for basic authentication (empty if none).
    pub(crate) password: String,
    /// The host (domain name, IPv4, or bracketed IPv6).
    pub(crate) host: String,
    /// The port number (defaults to scheme's default if not explicitly set).
    pub(crate) port: u16,
    /// Decoded path segments.
    pub(crate) path_segments: Vec<String>,
    /// Flattened query names and values (name0, value0, name1, value1, …).
    pub(crate) query_names_and_values: Vec<String>,
    /// The fragment component (decoded), or `None`.
    pub(crate) fragment: Option<String>,
}

impl HttpUrl {
    /// Create a new builder.
    pub fn builder() -> HttpUrlBuilder {
        HttpUrlBuilder::new()
    }

    /// Create a builder pre-populated with this URL's components.
    pub fn new_builder(&self) -> HttpUrlBuilder {
        let mut b = HttpUrlBuilder::new();
        b.scheme = Some(self.scheme);
        b.username = self.username.clone();
        b.password = self.password.clone();
        b.host = Some(self.host.clone());
        b.port = Some(self.port);
        b.path_segments = self.path_segments.clone();
        b.query_names_and_values = self.query_names_and_values.clone();
        b.fragment = self.fragment.clone();
        b
    }

    /// Returns the URL scheme.
    pub fn scheme(&self) -> Scheme {
        self.scheme
    }

    /// Returns `true` if the scheme is `https`.
    pub fn is_https(&self) -> bool {
        self.scheme == Scheme::Https
    }

    /// Returns the username (empty string if not present).
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns the password (empty string if not present).
    pub fn password(&self) -> &str {
        &self.password
    }

    /// Returns the host (domain, IPv4, or bracketed IPv6).
    pub fn host(&self) -> &str {
        &self.host
    }

    /// The explicit port, or the default port for the scheme if none was set.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// The port only if it differs from the scheme's default.
    pub fn explicit_port(&self) -> Option<u16> {
        if self.port == self.scheme.default_port() {
            None
        } else {
            Some(self.port)
        }
    }

    /// The decoded path segments.
    pub fn path_segments(&self) -> &[String] {
        &self.path_segments
    }

    /// The encoded path (with leading `/`), e.g. `/a/b/c`.
    pub fn encoded_path(&self) -> String {
        if self.path_segments.is_empty() {
            return "/".to_string();
        }
        let mut out = String::new();
        for seg in &self.path_segments {
            out.push('/');
            out.push_str(&percent::encode_path_segment(seg));
        }
        out
    }

    /// The decoded path as a string, e.g. `/a/b/c`.
    pub fn path(&self) -> String {
        if self.path_segments.is_empty() {
            return "/".to_string();
        }
        let mut out = String::new();
        for seg in &self.path_segments {
            out.push('/');
            out.push_str(seg);
        }
        out
    }

    /// Returns the query string (percent-encoded), or `None` if there are no parameters.
    pub fn encoded_query(&self) -> Option<String> {
        if self.query_names_and_values.is_empty() {
            return None;
        }
        let mut out = String::new();
        for i in (0..self.query_names_and_values.len()).step_by(2) {
            if !out.is_empty() {
                out.push('&');
            }
            out.push_str(&percent::encode_query(&self.query_names_and_values[i]));
            if i + 1 < self.query_names_and_values.len() {
                out.push('=');
                out.push_str(&percent::encode_query(&self.query_names_and_values[i + 1]));
            }
        }
        Some(out)
    }

    /// Returns the decoded query string, or `None`.
    pub fn query(&self) -> Option<String> {
        if self.query_names_and_values.is_empty() {
            return None;
        }
        let mut out = String::new();
        for i in (0..self.query_names_and_values.len()).step_by(2) {
            if !out.is_empty() {
                out.push('&');
            }
            out.push_str(&self.query_names_and_values[i]);
            if i + 1 < self.query_names_and_values.len() {
                out.push('=');
                out.push_str(&self.query_names_and_values[i + 1]);
            }
        }
        Some(out)
    }

    /// Get the first value for a query parameter name.
    pub fn query_parameter(&self, name: &str) -> Option<&str> {
        for i in (0..self.query_names_and_values.len()).step_by(2) {
            if self.query_names_and_values[i] == name {
                return if i + 1 < self.query_names_and_values.len() {
                    Some(&self.query_names_and_values[i + 1])
                } else {
                    Some("")
                };
            }
        }
        None
    }

    /// Get all values for a query parameter name.
    pub fn query_parameters(&self, name: &str) -> Vec<&str> {
        let mut result = Vec::new();
        for i in (0..self.query_names_and_values.len()).step_by(2) {
            if self.query_names_and_values[i] == name {
                result.push(if i + 1 < self.query_names_and_values.len() {
                    &self.query_names_and_values[i + 1]
                } else {
                    ""
                });
            }
        }
        result
    }

    /// Returns the set of unique query parameter names.
    pub fn query_parameter_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = Vec::new();
        for i in (0..self.query_names_and_values.len()).step_by(2) {
            let name = &self.query_names_and_values[i];
            if !names.contains(&name.as_str()) {
                names.push(name);
            }
        }
        names
    }

    /// Number of query parameters.
    pub fn query_size(&self) -> usize {
        self.query_names_and_values.len() / 2
    }

    /// The fragment (decoded), if any.
    pub fn fragment(&self) -> Option<&str> {
        self.fragment.as_deref()
    }

    /// The fragment (percent-encoded), if any.
    pub fn encoded_fragment(&self) -> Option<String> {
        self.fragment
            .as_ref()
            .map(|f| percent::encode_path_segment(f))
    }

    /// Resolve a relative URL against this one.
    ///
    /// This follows the algorithm from RFC 3986 Section 5.
    pub fn resolve(&self, link: &str) -> Result<HttpUrl> {
        // If the link is absolute, just parse it directly.
        if link.contains("://") {
            return HttpUrl::parse(link);
        }

        let link = link.trim();

        if link.is_empty() {
            return Ok(self.clone());
        }

        // Build a new URL starting from a clone of this one.
        let mut builder = self.new_builder();

        if let Some(authority_part) = link.strip_prefix("//") {
            // Protocol-relative: keep scheme, replace rest
            let (authority, pqf) = parse::split_authority(authority_part);
            let (uname, pwd, host_str, port) = parse::parse_authority(authority, &self.scheme)?;
            builder.host = Some(host::canonicalize_host(host_str)?);
            builder.username = uname;
            builder.password = pwd;
            builder.port = Some(port);
            if !pqf.is_empty() {
                parse::apply_path_query_fragment(&mut builder, pqf)?;
            } else {
                builder.path_segments.clear();
                builder.query_names_and_values.clear();
                builder.fragment = None;
            }
        } else if let Some(stripped) = link.strip_prefix('/') {
            // Absolute path: replace path, query, fragment
            parse::apply_path_query_fragment(&mut builder, stripped)?;
        } else if link.starts_with('?') {
            // Replace query and fragment
            builder.query_names_and_values.clear();
            parse::apply_path_query_fragment(&mut builder, link)?;
        } else if let Some(stripped) = link.strip_prefix('#') {
            // Replace fragment only
            let frag = percent::decode(stripped)
                .map_err(|_| HttpUrlError::InvalidPercentEncoding(link.to_string()))?;
            builder.fragment = Some(frag);
        } else {
            // Relative path: merge with existing path
            parse::merge_path(&mut builder, link)?;
            // Note: merge_path should handle query/fragment internally if present
        }

        builder.build()
    }

    /// Compute the top private domain (eTLD+1) of this URL's host.
    ///
    /// Returns `None` if the host is an IP address or does not have enough
    /// domain labels. This is a simplified version; a full implementation
    /// would use the Public Suffix List.
    pub fn top_private_domain(&self) -> Option<String> {
        if !host::is_domain(&self.host) {
            return None;
        }
        let labels: Vec<&str> = self.host.split('.').collect();
        if labels.len() < 2 {
            return None;
        }
        // Simplified: assume the last two labels form the private domain.
        // A proper implementation would consult the PSL.
        let domain = format!("{}.{}", labels[labels.len() - 2], labels[labels.len() - 1]);
        Some(domain)
    }

    /// The full URL string (percent-encoded).
    pub fn to_url_string(&self) -> String {
        let mut out = String::new();
        out.push_str(self.scheme.as_str());
        out.push_str("://");

        if !self.username.is_empty() || !self.password.is_empty() {
            out.push_str(&percent::encode_user_info(&self.username));
            if !self.password.is_empty() {
                out.push(':');
                out.push_str(&percent::encode_user_info(&self.password));
            }
            out.push('@');
        }

        out.push_str(&self.host);

        if self.port() != self.scheme.default_port() {
            out.push(':');
            let _ = write!(out, "{}", self.port());
        }

        out.push_str(&self.encoded_path());

        if let Some(q) = self.encoded_query() {
            out.push('?');
            out.push_str(&q);
        }

        if let Some(f) = &self.fragment {
            out.push('#');
            out.push_str(&percent::encode_path_segment(f));
        }

        out
    }
}

impl fmt::Display for HttpUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_url_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoded_query() {
        let url = HttpUrl::builder()
            .scheme(Scheme::Http)
            .host("example.com")
            .add_query_parameter("name", "hello world")
            .build()
            .unwrap();
        assert_eq!(url.encoded_query().unwrap(), "name=hello%20world");
    }

    #[test]
    fn test_multiple_query_values() {
        let url = HttpUrl::parse("http://example.com/?a=1&a=2").unwrap();
        assert_eq!(url.query_parameter("a"), Some("1"));
        assert_eq!(url.query_parameters("a"), vec!["1", "2"]);
    }

    #[test]
    fn test_multi_query_parameter_names() {
        let url = HttpUrl::parse("http://example.com/?a=1&b=2&a=3").unwrap();
        let names = url.query_parameter_names();
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));
    }

    #[test]
    fn test_resolve_absolute() {
        let base = HttpUrl::parse("http://example.com/a/b").unwrap();
        let resolved = base.resolve("http://other.com/c").unwrap();
        assert_eq!(resolved.host(), "other.com");
        assert_eq!(resolved.path(), "/c");
    }

    #[test]
    fn test_resolve_relative() {
        let base = HttpUrl::parse("http://example.com/a/b").unwrap();
        let resolved = base.resolve("c").unwrap();
        assert_eq!(resolved.path(), "/a/c");
    }

    #[test]
    fn test_resolve_absolute_path() {
        let base = HttpUrl::parse("http://example.com/a/b").unwrap();
        let resolved = base.resolve("/c/d").unwrap();
        assert_eq!(resolved.path(), "/c/d");
    }

    #[test]
    fn test_resolve_protocol_relative() {
        let base = HttpUrl::parse("http://example.com/a/b").unwrap();
        let resolved = base.resolve("//other.com/c").unwrap();
        assert_eq!(resolved.scheme(), "http");
        assert_eq!(resolved.host(), "other.com");
        assert_eq!(resolved.path(), "/c");
    }

    #[test]
    fn test_resolve_query_only() {
        let base = HttpUrl::parse("http://example.com/a/b?old=1").unwrap();
        let resolved = base.resolve("?new=2").unwrap();
        assert_eq!(resolved.query_parameter("new"), Some("2"));
        assert_eq!(resolved.query_parameter("old"), None);
    }

    #[test]
    fn test_resolve_fragment_only() {
        let base = HttpUrl::parse("http://example.com/a/b#old").unwrap();
        let resolved = base.resolve("#new").unwrap();
        assert_eq!(resolved.fragment(), Some("new"));
    }

    #[test]
    fn test_to_string_no_default_port() {
        let url = HttpUrl::parse("http://example.com/").unwrap();
        assert_eq!(url.to_string(), "http://example.com/");
    }

    #[test]
    fn test_to_string_with_port() {
        let url = HttpUrl::parse("http://example.com:8080/").unwrap();
        assert_eq!(url.to_string(), "http://example.com:8080/");
    }

    #[test]
    fn test_to_string_with_userinfo() {
        let url = HttpUrl::parse("http://user:pass@example.com/").unwrap();
        assert_eq!(url.to_string(), "http://user:pass@example.com/");
    }

    #[test]
    fn test_top_private_domain() {
        let url = HttpUrl::parse("http://www.example.com/path").unwrap();
        assert_eq!(url.top_private_domain(), Some("example.com".to_string()));
    }

    #[test]
    fn test_top_private_domain_ip() {
        let url = HttpUrl::parse("http://192.168.1.1/").unwrap();
        assert_eq!(url.top_private_domain(), None);
    }

    #[test]
    fn test_new_builder_from_url() {
        let url = HttpUrl::parse("http://example.com/path?q=1").unwrap();
        let builder = url.new_builder();
        let url2 = builder.build().unwrap();
        assert_eq!(url, url2);
    }
}
