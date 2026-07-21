//! Crate-local percent-decoding helper.
//!
//! Parsing and percent-encoding of URLs is delegated to the `url` crate.
//! The builder stores path segments and fragments in *decoded* form, so the
//! only local helper needed is decoding percent-encoding when populating a
//! builder from an existing [`url::Url`][crate::HttpUrl].
//!
//! Path segments and fragments do not use the `+`-for-space convention of
//! `application/x-www-form-urlencoded`, so this decoder treats `+` literally.

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// Percent-decode `input`, leaving any invalid `%` sequence untouched.
///
/// This is intentionally lenient: `url::Url` only ever stores valid
/// percent-encoding, so in practice malformed sequences never appear here.
pub(crate) fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                out.push((hi << 4) | lo);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| input.to_string())
}

fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'A'..=b'F' => Some(c - b'A' + 10),
        b'a'..=b'f' => Some(c - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_plain() {
        assert_eq!(percent_decode("hello"), "hello");
    }

    #[test]
    fn decode_percent() {
        assert_eq!(percent_decode("hello%20world"), "hello world");
        assert_eq!(percent_decode("%E4%B8%AD%E6%96%87"), "中文");
    }

    #[test]
    fn decode_plus_literal() {
        // Path segments don't use `+` for space.
        assert_eq!(percent_decode("a+b"), "a+b");
    }

    #[test]
    fn decode_lenient_on_invalid() {
        assert_eq!(percent_decode("%ZZ"), "%ZZ");
        assert_eq!(percent_decode("%"), "%");
    }
}
