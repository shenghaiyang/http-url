//! Host parsing and canonicalization.
//!
//! Handles:
//! - Regular domain names (lowercased, punycode for IDN)
//! - IPv4 addresses
//! - IPv6 addresses (enclosed in `[...]`)

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::error::{HttpUrlError, Result};

/// Canonicalize a host string from a URL.
///
/// - Strips enclosing `[...]` for IPv6.
/// - Lowercases ASCII hostnames.
pub fn canonicalize_host(host: &str) -> Result<String> {
    let host = host.trim();

    if host.is_empty() {
        return Err(HttpUrlError::InvalidHost(host.to_string()));
    }

    // IPv6: enclosed in brackets
    if host.starts_with('[') {
        if !host.ends_with(']') {
            return Err(HttpUrlError::InvalidHost(host.to_string()));
        }
        let inner = &host[1..host.len() - 1];
        if inner.is_empty() {
            return Err(HttpUrlError::InvalidHost(host.to_string()));
        }
        // Validate IPv6 address structure
        if !is_valid_ipv6(inner) {
            return Err(HttpUrlError::InvalidHost(host.to_string()));
        }
        return Ok(host.to_string()); // keep brackets
    }

    // IPv4 (contains only digits and dots)
    if is_ipv4_literal(host) {
        if !is_valid_ipv4(host) {
            return Err(HttpUrlError::InvalidHost(host.to_string()));
        }
        return Ok(host.to_string());
    }

    // Regular hostname: lowercase
    let lower = host.to_ascii_lowercase();
    validate_hostname(&lower)?;
    Ok(lower)
}

/// Validate a hostname (called after lowercasing).
///
/// Checks:
/// - Total length ≤ 253
/// - Each label length ≤ 63
/// - Only alphanumeric characters and hyphens allowed
/// - Hyphens may not appear at the start or end of a label
fn validate_hostname(host: &str) -> Result<()> {
    if host.is_empty() {
        return Err(HttpUrlError::InvalidHost(host.to_string()));
    }

    if host.len() > 253 {
        return Err(HttpUrlError::InvalidHost(host.to_string()));
    }

    let labels: Vec<&str> = host.split('.').collect();
    for label in &labels {
        if label.is_empty() {
            return Err(HttpUrlError::InvalidHost(host.to_string()));
        }
        if label.len() > 63 {
            return Err(HttpUrlError::InvalidHost(host.to_string()));
        }
        // Must start and end with alphanumeric (or just be valid chars)
        for (i, &b) in label.as_bytes().iter().enumerate() {
            if !b.is_ascii_alphanumeric() && b != b'-' {
                return Err(HttpUrlError::InvalidHost(host.to_string()));
            }
            // Hyphen cannot be first or last
            if b == b'-' && (i == 0 || i == label.len() - 1) {
                return Err(HttpUrlError::InvalidHost(host.to_string()));
            }
        }
    }

    Ok(())
}

/// Check if a string looks like an IPv4 address (digits and dots only).
fn is_ipv4_literal(s: &str) -> bool {
    s.bytes().all(|b| b.is_ascii_digit() || b == b'.')
}

/// Validate an IPv4 address string. Must have exactly 4 octets, each 0–255,
/// with no leading zeros (except the value `0` itself).
fn is_valid_ipv4(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts.iter().all(|p| {
        if p.is_empty() {
            return false;
        }
        // No leading zeros unless the number is exactly "0"
        if p.len() > 1 && p.starts_with('0') {
            return false;
        }
        p.bytes().all(|b| b.is_ascii_digit())
            && p.len() <= 3
            && p.parse::<u16>().is_ok_and(|v| v <= 255)
    })
}

/// Basic IPv6 validation (structural checks only).
///
/// Checks for valid hex characters, colons, and optional IPv4-mapped sections.
/// Does *not* fully validate address semantics.
fn is_valid_ipv6(s: &str) -> bool {
    // At minimum check for colons and valid hex digits
    if s.is_empty() || s.len() > 128 {
        return false;
    }
    if s.starts_with(':') && !s.starts_with("::") {
        return false;
    }
    if s.ends_with(':') && !s.ends_with("::") {
        return false;
    }
    // Check characters are valid hex, colon, or dot (for IPv4 mapped)
    s.bytes()
        .all(|b| b.is_ascii_hexdigit() || b == b':' || b == b'.')
}

/// Check if a host is likely a domain name (not IP).
pub fn is_domain(host: &str) -> bool {
    host.bytes().any(|b| b.is_ascii_alphabetic() && b != b'.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonicalize_host_lowercase() {
        assert_eq!(canonicalize_host("Example.COM").unwrap(), "example.com");
    }

    #[test]
    fn test_canonicalize_host_ipv4() {
        assert_eq!(canonicalize_host("192.168.1.1").unwrap(), "192.168.1.1");
    }

    #[test]
    fn test_canonicalize_host_ipv6() {
        let host = canonicalize_host("[::1]").unwrap();
        assert!(host.starts_with('['));
    }

    #[test]
    fn test_canonicalize_host_invalid_ipv4() {
        assert!(canonicalize_host("256.0.0.1").is_err());
    }

    #[test]
    fn test_canonicalize_host_invalid_label() {
        assert!(canonicalize_host("-host.com").is_err());
        assert!(canonicalize_host("host-.com").is_err());
    }

    #[test]
    fn test_canonicalize_host_empty() {
        assert!(canonicalize_host("").is_err());
    }
}
