use alloc::string::ToString;
use alloc::vec::Vec;
use core::str::FromStr;

use crate::error::Result;
use crate::util::{host, parse, percent};
use crate::{HttpUrl, HttpUrlError, Scheme};

impl HttpUrl {
    /// Parse a URL string into an `HttpUrl`.
    ///
    /// The URL must have an `http://` or `https://` scheme.
    pub fn parse(url: &str) -> Result<Self> {
        if url.is_empty() {
            return Err(HttpUrlError::EmptyUrl);
        }

        let url = url.trim();

        // 1. Find scheme
        let scheme_end = url.find("://").ok_or(HttpUrlError::MissingScheme)?;
        let scheme = Scheme::from_str(&url[..scheme_end])?;
        let rest = &url[scheme_end + 3..];

        if rest.is_empty() {
            return Err(HttpUrlError::MissingHost);
        }

        // 2. Split authority, path, query, fragment
        //    Find the start of path (first '/' after authority), or query '?', or fragment '#'
        let (authority, path_query_fragment) = parse::split_authority(rest);

        // 3. Parse userinfo@host:port from authority
        let (username, password, host_str, port) = parse::parse_authority(authority, &scheme)?;

        // 4. Parse host
        let host = host::canonicalize_host(host_str)?;

        // 5. Split path?query#fragment
        let (path_str, query_str, fragment_str) =
            parse::split_path_query_fragment(path_query_fragment);

        // 6. Parse path segments
        let path_segments = parse::parse_path_segments(path_str)?;

        // 7. Parse query parameters
        let query_names_and_values = if let Some(q) = query_str {
            parse::parse_query_string(q)?
        } else {
            Vec::new()
        };

        // 8. Fragment (percent-decoded)
        let fragment = match fragment_str {
            Some(f) => Some(
                percent::decode(f)
                    .map_err(|_| HttpUrlError::InvalidPercentEncoding(f.to_string()))?,
            ),
            None => None,
        };

        Ok(Self {
            scheme,
            username,
            password,
            host,
            port,
            path_segments,
            query_names_and_values,
            fragment,
        })
    }
}

impl FromStr for HttpUrl {
    type Err = HttpUrlError;

    fn from_str(s: &str) -> Result<Self> {
        HttpUrl::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let url = HttpUrl::parse("http://example.com").unwrap();
        assert_eq!(url.scheme(), "http");
        assert_eq!(url.host(), "example.com");
        assert_eq!(url.port(), 80);
        assert_eq!(url.path(), "/");
    }

    #[test]
    fn test_parse_https() {
        let url = HttpUrl::parse("https://example.com/path").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.port(), 443);
        assert_eq!(url.path(), "/path");
    }

    #[test]
    fn test_parse_with_port() {
        let url = HttpUrl::parse("http://example.com:8080/").unwrap();
        assert_eq!(url.port(), 8080);
        assert_eq!(url.explicit_port(), Some(8080));
    }

    #[test]
    fn test_parse_default_port_explicit() {
        let url = HttpUrl::parse("http://example.com:80/").unwrap();
        assert_eq!(url.port(), 80);
        assert_eq!(url.explicit_port(), None);
    }

    #[test]
    fn test_parse_with_userinfo() {
        let url = HttpUrl::parse("http://user:pass@example.com/").unwrap();
        assert_eq!(url.username(), "user");
        assert_eq!(url.password(), "pass");
    }

    #[test]
    fn test_parse_with_username_only() {
        let url = HttpUrl::parse("http://user@example.com/").unwrap();
        assert_eq!(url.username(), "user");
        assert_eq!(url.password(), "");
    }

    #[test]
    fn test_parse_with_query() {
        let url = HttpUrl::parse("http://example.com/?a=1&b=2").unwrap();
        assert_eq!(url.query_parameter("a"), Some("1"));
        assert_eq!(url.query_parameter("b"), Some("2"));
        assert_eq!(url.query_size(), 2);
    }

    #[test]
    fn test_parse_with_fragment() {
        let url = HttpUrl::parse("http://example.com/#section").unwrap();
        assert_eq!(url.fragment(), Some("section"));
    }

    #[test]
    fn test_parse_encoded_path() {
        let url = HttpUrl::parse("http://example.com/hello%20world").unwrap();
        assert_eq!(url.path(), "/hello world");
        assert_eq!(url.encoded_path(), "/hello%20world");
    }

    #[test]
    fn test_parse_ipv6() {
        let url = HttpUrl::parse("http://[::1]:8080/path").unwrap();
        assert_eq!(url.host(), "[::1]");
        assert_eq!(url.port(), 8080);
    }

    #[test]
    fn test_parse_errors() {
        assert!(HttpUrl::parse("").is_err());
        assert!(HttpUrl::parse("ftp://example.com").is_err());
        assert!(HttpUrl::parse("http://").is_err());
    }

    #[test]
    fn test_path_canonicalization() {
        let url = HttpUrl::parse("http://example.com/a/b/../c/./d").unwrap();
        assert_eq!(url.path(), "/a/c/d");
    }

    #[test]
    fn test_empty_query() {
        let url = HttpUrl::parse("http://example.com/?").unwrap();
        assert_eq!(url.query(), None);
        assert_eq!(url.query_size(), 0);
    }

    #[test]
    fn test_unicode_path() {
        let url = HttpUrl::parse("http://example.com/%E4%B8%AD%E6%96%87").unwrap();
        assert_eq!(url.path(), "/中文");
    }

    #[test]
    fn test_backslash_in_path() {
        let url = HttpUrl::parse("http://example.com/a\\b\\c").unwrap();
        assert_eq!(url.path(), "/a/b/c");

        let url = HttpUrl::parse("http://example.com/a\\b/../c").unwrap();
        assert_eq!(url.path(), "/a/c");
    }

    #[test]
    fn test_port_from_str() {
        let url: HttpUrl = "http://example.com:1234/".parse().unwrap();
        assert_eq!(url.port(), 1234);
    }

    #[test]
    fn test_parse_roundtrip() {
        let inputs = [
            "http://example.com/",
            "https://example.com/path/to?q=1&r=2#frag",
            "http://user:pass@host.com:8080/a/b/c",
            "http://[::1]:9090/path",
            "http://example.com/hello%20world",
        ];
        for input in &inputs {
            let url = HttpUrl::parse(input).unwrap();
            let output = url.to_url_string();
            let reparsed = HttpUrl::parse(&output).unwrap();
            assert_eq!(url, reparsed, "roundtrip failed for: {}", input);
        }
    }
}
