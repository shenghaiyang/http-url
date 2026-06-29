//! Percent-encoding and decoding for URL components (RFC 3986).

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

#[cfg(test)]
use alloc::vec;

// ── Character classification helpers ──────────────────────────────

/// Unreserved characters per RFC 3986: `A-Z`, `a-z`, `0-9`, `-`, `.`, `_`, `~`
fn is_unreserved(c: u8) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, b'-' | b'.' | b'_' | b'~')
}

/// Sub-delimiters per RFC 3986: `!`, `$`, `&`, `'`, `(`, `)`, `*`, `+`, `,`, `;`, `=`
fn is_sub_delim(c: u8) -> bool {
    matches!(
        c,
        b'!' | b'$' | b'&' | b'\'' | b'(' | b')' | b'*' | b'+' | b',' | b';' | b'='
    )
}

/// Characters allowed in a path segment (pchar) = unreserved + sub-delims + `:` + `@`
fn is_pchar(c: u8) -> bool {
    is_unreserved(c) || is_sub_delim(c) || matches!(c, b':' | b'@')
}

/// Characters that do *not* need encoding inside a path segment.
fn is_path_segment_allowed(c: u8) -> bool {
    is_pchar(c)
}

/// Characters allowed inside a query component (pchar + `/` + `?`).
fn is_query_allowed(c: u8) -> bool {
    is_pchar(c) || matches!(c, b'/' | b'?')
}

/// Characters allowed inside a user-info component (unreserved + sub-delims + `:`).
fn is_user_info_allowed(c: u8) -> bool {
    is_unreserved(c) || is_sub_delim(c) || c == b':'
}

// ── Public API ────────────────────────────────────────────────────

/// Percent-encode a string for use as a path segment.
///
/// Characters that are *not* allowed in a path segment are
/// percent-encoded.
pub fn encode_path_segment(input: &str) -> String {
    percent_encode(input, is_path_segment_allowed)
}

/// Percent-encode a string for use in a query component.
///
/// Reserves `+` so it isn't treated as a space later.
pub fn encode_query(input: &str) -> String {
    percent_encode(input, is_query_allowed)
}

/// Percent-encode a query parameter *name* or *value*
/// in the form-encoded style (like `application/x-www-form-urlencoded`).
///
/// Space becomes `+`, other disallowed chars are percent-encoded.
#[allow(dead_code)]
pub fn encode_form_query(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for &b in input.as_bytes() {
        match b {
            b' ' => out.push('+'),
            _ if is_unreserved(b) => out.push(b as char),
            _ => percent_encode_byte(b, &mut out),
        }
    }
    out
}

/// Percent-encode a string for use in the user-info part (username / password).
pub fn encode_user_info(input: &str) -> String {
    percent_encode(input, is_user_info_allowed)
}

/// Decode a percent-encoded string.
///
/// Returns `None` if an invalid percent sequence (`%` not followed by
/// two hex digits) is encountered.
pub fn decode(input: &str) -> Result<String, String> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hi = hex_val(bytes[i + 1]).ok_or_else(|| {
                    format!(
                        "invalid percent-encoding at byte {}: '%{}'",
                        i,
                        core::str::from_utf8(&bytes[i..(i + 3).min(bytes.len())]).unwrap_or("??")
                    )
                })?;
                let lo = hex_val(bytes[i + 2]).ok_or_else(|| {
                    format!(
                        "invalid percent-encoding at byte {}: '%{}'",
                        i,
                        core::str::from_utf8(&bytes[i..(i + 3).min(bytes.len())]).unwrap_or("??")
                    )
                })?;
                out.push((hi << 4) | lo);
                i += 3;
                continue;
            }
            b'%' => {
                let tail =
                    core::str::from_utf8(&bytes[i..(i + 3).min(bytes.len())]).unwrap_or("??");
                return Err(format!(
                    "truncated percent-encoding at byte {}: '{}'",
                    i, tail
                ));
            }
            b'+' => out.push(b' '), // application/x-www-form-urlencoded convention
            c => out.push(c),
        }
        i += 1;
    }
    String::from_utf8(out).map_err(|_| "invalid UTF-8 after decoding".to_string())
}

// ── Internal helpers ──────────────────────────────────────────────

fn percent_encode(input: &str, allowed: fn(u8) -> bool) -> String {
    let bytes = input.as_bytes();
    // Pre-allocate a reasonable capacity
    let mut out = String::with_capacity(bytes.len());
    for &b in bytes {
        if allowed(b) {
            out.push(b as char);
        } else {
            percent_encode_byte(b, &mut out);
        }
    }
    out
}

fn percent_encode_byte(b: u8, out: &mut String) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    out.push('%');
    out.push(HEX[(b >> 4) as usize] as char);
    out.push(HEX[(b & 0x0F) as usize] as char);
}

fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'A'..=b'F' => Some(c - b'A' + 10),
        b'a'..=b'f' => Some(c - b'a' + 10),
        _ => None,
    }
}

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_path_segment_simple() {
        assert_eq!(encode_path_segment("hello"), "hello");
    }

    #[test]
    fn test_encode_path_segment_space() {
        assert_eq!(encode_path_segment("hello world"), "hello%20world");
    }

    #[test]
    fn test_encode_path_segment_unicode() {
        assert_eq!(encode_path_segment("中文"), "%E4%B8%AD%E6%96%87");
    }

    #[test]
    fn test_encode_path_segment_special() {
        // Reserved characters are encoded
        assert_eq!(encode_path_segment("a/b"), "a%2Fb");
        assert_eq!(encode_path_segment("a?b"), "a%3Fb");
        assert_eq!(encode_path_segment("a#b"), "a%23b");
    }

    #[test]
    fn test_decode_simple() {
        assert_eq!(decode("hello").unwrap(), "hello");
    }

    #[test]
    fn test_decode_percent() {
        assert_eq!(decode("hello%20world").unwrap(), "hello world");
    }

    #[test]
    fn test_decode_plus() {
        assert_eq!(decode("hello+world").unwrap(), "hello world");
    }

    #[test]
    fn test_decode_unicode() {
        assert_eq!(decode("%E4%B8%AD%E6%96%87").unwrap(), "中文");
    }

    #[test]
    fn test_decode_invalid() {
        assert!(decode("%XX").is_err());
        assert!(decode("%").is_err());
        assert!(decode("%G").is_err());
    }

    #[test]
    fn test_roundtrip() {
        let cases = vec!["hello", "hello world", "a/b?c#d", "你好", "foo%bar"];
        for s in &cases {
            let encoded = encode_path_segment(s);
            let decoded = decode(&encoded).unwrap();
            assert_eq!(&decoded, s);
        }
    }

    #[test]
    fn test_encode_form_query() {
        assert_eq!(encode_form_query("hello world"), "hello+world");
        assert_eq!(encode_form_query("a & b"), "a+%26+b");
    }

    #[test]
    fn test_encode_user_info() {
        assert_eq!(encode_user_info("user"), "user");
        assert_eq!(encode_user_info("user@host"), "user%40host");
        assert_eq!(encode_user_info("user:pass"), "user:pass");
    }
}
