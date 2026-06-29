use core::fmt;
use core::str::FromStr;

use alloc::string::ToString;

use crate::HttpUrlError;
use crate::error::Result;

/// Default port for HTTP URLs.
const DEFAULT_HTTP_PORT: u16 = 80;

/// Default port for HTTPS URLs.
const DEFAULT_HTTPS_PORT: u16 = 443;

/// The URL scheme – only `http` and `https` are supported.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Scheme {
    /// `http://`
    Http,
    /// `https://`
    Https,
}

impl Scheme {
    /// Returns the scheme as a static string (`"http"` or `"https"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
        }
    }

    /// Returns the default port for this scheme (`80` for http, `443` for https).
    pub fn default_port(&self) -> u16 {
        match self {
            Self::Http => DEFAULT_HTTP_PORT,
            Self::Https => DEFAULT_HTTPS_PORT,
        }
    }
}

impl FromStr for Scheme {
    type Err = HttpUrlError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "http" => Ok(Self::Http),
            "https" => Ok(Self::Https),
            other => Err(HttpUrlError::UnsupportedScheme(other.to_string())),
        }
    }
}

impl fmt::Display for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for Scheme {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<&str> for Scheme {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Scheme::Http), "http");
        assert_eq!(format!("{}", Scheme::Https), "https");
    }

    #[test]
    fn test_as_ref() {
        assert_eq!(Scheme::Http.as_ref(), "http");
        assert_eq!(Scheme::Https.as_ref(), "https");
    }

    #[test]
    fn test_from_str() {
        assert_eq!("http".parse::<Scheme>().unwrap(), Scheme::Http);
        assert_eq!("https".parse::<Scheme>().unwrap(), Scheme::Https);
        assert!("ftp".parse::<Scheme>().is_err());
    }

    #[test]
    fn test_default_port() {
        assert_eq!(Scheme::Http.default_port(), 80);
        assert_eq!(Scheme::Https.default_port(), 443);
    }
}
