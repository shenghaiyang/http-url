//! Internal URL parsing helpers.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::builder::HttpUrlBuilder;
use crate::error::{HttpUrlError, Result};
use crate::scheme::Scheme;
use crate::util::percent;

/// Split `rest` (the part after `://`) into authority and the rest.
///
/// Authority is everything up to the first `/`, `?`, or `#`.
pub(crate) fn split_authority(rest: &str) -> (&str, &str) {
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    (&rest[..end], &rest[end..])
}

/// Split path?query#fragment into its three components.
pub(crate) fn split_path_query_fragment(pqf: &str) -> (&str, Option<&str>, Option<&str>) {
    let pqf = if pqf.is_empty() { "/" } else { pqf };

    let frag_start = pqf.find('#');
    let (before_frag, frag) = match frag_start {
        Some(pos) => (&pqf[..pos], Some(&pqf[pos + 1..])),
        None => (pqf, None),
    };

    let query_start = before_frag.find('?');
    let (path, query) = match query_start {
        Some(pos) => (&before_frag[..pos], Some(&before_frag[pos + 1..])),
        None => (before_frag, None),
    };

    (path, query, frag)
}

/// Parse a path string into decoded path segments.
///
/// Backslashes (`\`) are converted to forward slashes for robustness.
pub(crate) fn parse_path_segments(path: &str) -> Result<Vec<String>> {
    let path = if path.is_empty() { "/" } else { path };

    // Normalize backslashes to forward slashes.
    let path = path.replace('\\', "/");

    let mut segments = Vec::new();

    for segment in path.split('/') {
        if segment.is_empty() {
            continue;
        }
        let decoded = percent::decode(segment)
            .map_err(|_| HttpUrlError::InvalidPercentEncoding(segment.to_string()))?;
        segments.push(decoded);
    }

    // Canonicalize: remove `.` and `..`
    canonicalize_path_segments(&mut segments);

    Ok(segments)
}

/// Remove `.` and `..` segments from a path (in place).
pub(crate) fn canonicalize_path_segments(segments: &mut Vec<String>) {
    let mut i = 0;
    while i < segments.len() {
        if segments[i] == "." {
            segments.remove(i);
        } else if segments[i] == ".." {
            if i > 0 {
                segments.remove(i);
                segments.remove(i - 1);
                i -= 1;
            } else {
                segments.remove(i);
            }
        } else {
            i += 1;
        }
    }
}

/// Parse a query string like `a=1&b=2&c` into a flat list of decoded name/value pairs.
pub(crate) fn parse_query_string(query: &str) -> Result<Vec<String>> {
    let mut parts = Vec::new();
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        if let Some(eq_pos) = pair.find('=') {
            let name = percent::decode(&pair[..eq_pos])
                .map_err(|_| HttpUrlError::InvalidPercentEncoding(pair[..eq_pos].to_string()))?;
            let value = percent::decode(&pair[eq_pos + 1..]).map_err(|_| {
                HttpUrlError::InvalidPercentEncoding(pair[eq_pos + 1..].to_string())
            })?;
            parts.push(name);
            parts.push(value);
        } else {
            let name = percent::decode(pair)
                .map_err(|_| HttpUrlError::InvalidPercentEncoding(pair.to_string()))?;
            parts.push(name);
            parts.push(String::new());
        }
    }
    Ok(parts)
}

/// Parse authority into (username, password, host, port).
pub(crate) fn parse_authority<'a>(
    authority: &'a str,
    default_scheme: &Scheme,
) -> Result<(String, String, &'a str, u16)> {
    let default_port = default_scheme.default_port();

    // Split userinfo@host:port
    let (userinfo, host_port) = if let Some(at_pos) = authority.rfind('@') {
        let userinfo = &authority[..at_pos];
        let host_port = &authority[at_pos + 1..];
        (Some(userinfo), host_port)
    } else {
        (None, authority)
    };

    // Parse userinfo
    let (username, password) = if let Some(ui) = userinfo {
        if let Some(colon_pos) = ui.find(':') {
            let uname = percent::decode(&ui[..colon_pos])
                .map_err(|_| HttpUrlError::InvalidPercentEncoding(ui[..colon_pos].to_string()))?;
            let pwd = percent::decode(&ui[colon_pos + 1..]).map_err(|_| {
                HttpUrlError::InvalidPercentEncoding(ui[colon_pos + 1..].to_string())
            })?;
            (uname, pwd)
        } else {
            let uname = percent::decode(ui)
                .map_err(|_| HttpUrlError::InvalidPercentEncoding(ui.to_string()))?;
            (uname, String::new())
        }
    } else {
        (String::new(), String::new())
    };

    // Split host and port
    if host_port.is_empty() {
        return Err(HttpUrlError::MissingHost);
    }

    // IPv6 host is enclosed in brackets: [::1]:8080
    let (host_str, port) = if host_port.starts_with('[') {
        let close_bracket = host_port
            .find(']')
            .ok_or_else(|| HttpUrlError::InvalidHost(host_port.to_string()))?;
        let ipv6_host = &host_port[..close_bracket + 1];
        let after = &host_port[close_bracket + 1..];
        if after.is_empty() {
            (ipv6_host, default_port)
        } else if let Some(port_str) = after.strip_prefix(':') {
            let port = port_str
                .parse::<u16>()
                .map_err(|_| HttpUrlError::InvalidPort(port_str.to_string()))?;
            (ipv6_host, port)
        } else {
            return Err(HttpUrlError::InvalidHost(host_port.to_string()));
        }
    } else if let Some(colon_pos) = host_port.rfind(':') {
        let host_str = &host_port[..colon_pos];
        let port_str = &host_port[colon_pos + 1..];
        if port_str.is_empty() {
            return Err(HttpUrlError::InvalidPort("".to_string()));
        }
        let port = port_str
            .parse::<u16>()
            .map_err(|_| HttpUrlError::InvalidPort(port_str.to_string()))?;
        (host_str, port)
    } else {
        (host_port, default_port)
    };

    Ok((username, password, host_str, port))
}

/// Apply a path/query/fragment string to a builder (used during resolve).
pub(crate) fn apply_path_query_fragment(builder: &mut HttpUrlBuilder, s: &str) -> Result<()> {
    let (path_str, query_str, frag_str) = split_path_query_fragment(s);

    // Set path
    let segments = parse_path_segments(path_str)?;
    builder.path_segments = segments;

    // Set query
    if let Some(q) = query_str {
        builder.query_names_and_values = parse_query_string(q)?;
    }

    // Set fragment
    if let Some(f) = frag_str {
        let decoded =
            percent::decode(f).map_err(|_| HttpUrlError::InvalidPercentEncoding(f.to_string()))?;
        builder.fragment = Some(decoded);
    }

    Ok(())
}

/// Merge a relative path into an existing builder's path (RFC 3986 Section 5.2.3).
pub(crate) fn merge_path(builder: &mut HttpUrlBuilder, relative: &str) -> Result<()> {
    // If the relative path contains query or fragment, split them off
    let (rel_path, query_str, frag_str) = split_path_query_fragment(relative);

    // Merge path: take the base path, remove last segment, append relative
    if !builder.path_segments.is_empty() {
        builder.path_segments.pop();
    }
    for segment in rel_path.split('/') {
        if segment.is_empty() {
            continue;
        }
        let decoded = percent::decode(segment)
            .map_err(|_| HttpUrlError::InvalidPercentEncoding(segment.to_string()))?;
        builder.path_segments.push(decoded);
    }

    // Canonicalize
    canonicalize_path_segments(&mut builder.path_segments);

    // Apply query
    if let Some(q) = query_str {
        builder.query_names_and_values = parse_query_string(q)?;
    }

    // Apply fragment
    if let Some(f) = frag_str {
        let decoded =
            percent::decode(f).map_err(|_| HttpUrlError::InvalidPercentEncoding(f.to_string()))?;
        builder.fragment = Some(decoded);
    }

    Ok(())
}
