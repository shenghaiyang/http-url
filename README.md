# HttpUrl

[![Crates.io][crates-badge]][crates-url]

[crates-badge]: https://img.shields.io/crates/v/http-url.svg
[crates-url]: https://crates.io/crates/http-url

Parsing and building HTTP/HTTPS URLs in Rust.

## Features

- Fluent HTTP/HTTPS URL builder
- RFC 3986–compliant URL parsing
- `no_std` support

## Quick Start

```toml
[dependencies]
http-url = "0.1"
```

```rust
use http_url::{HttpUrl, Scheme};

// Parse
let url = HttpUrl::parse("https://user:pass@example.com:8080/path/to?a=1&b=2#frag").unwrap();

assert_eq!(url.scheme(), "https");
assert_eq!(url.host(), "example.com");
assert_eq!(url.port(), 8080);
assert_eq!(url.username(), "user");
assert_eq!(url.password(), "pass");
assert_eq!(url.query_parameter("a"), Some("1"));
assert_eq!(url.fragment(), Some("frag"));

// Build
let url = HttpUrl::builder()
    .scheme(Scheme::Https)
    .host("api.example.com")
    .add_path_segment("v1")
    .add_path_segment("users")
    .add_query_parameter("id", "42")
    .build()
    .unwrap();

assert_eq!(url.to_string(), "https://api.example.com/v1/users?id=42");
```

### Builder API

```rust
HttpUrl::builder()
    .scheme(Scheme::Https)              // Scheme::Http | Scheme::Https
    .username("user")                   // optional
    .password("pass")                   // optional
    .host("example.com")                // domain, IPv4, or [IPv6]
    .port(8080)                         // optional, uses scheme default
    .add_path_segment("api")            // decoded segment (auto-encoded)
    .add_path_segment("v1")
    .add_path_segments( & ["a", "b"])   // multiple segments at once
    .set_path("/a/b/c")                 // replace entire path (decoded)
    .set_encoded_path("/a%20b/c")       // replace with pre-encoded path
    .add_query_parameter("key", "val")  // decoded form
    .add_encoded_query_parameter("k%20y", "v%20l")  // pre-encoded
    .remove_query_parameter("key")      // remove all with this name
    .clear_query_parameters()           // remove all
    .fragment("section")                // decoded fragment
    .encoded_fragment("sec%20tion")     // pre-encoded
    .remove_fragment()                  // clear fragment
    .build()
    .unwrap();
```

### Query Parameters

```rust
let url = HttpUrl::parse("http://example.com/?a=1&a=2&b=3").unwrap();

// First value
assert_eq!(url.query_parameter("a"), Some("1"));

// All values
assert_eq!(url.query_parameters("a"), vec!["1", "2"]);

// Missing name
assert_eq!(url.query_parameter("z"), None);

// Unique names
let names = url.query_parameter_names();
assert_eq!(names.len(), 2);
assert!(names.contains(&"a"));
assert!(names.contains(&"b"));

// Count
assert_eq!(url.query_size(), 3);
```

### Relative URL Resolution

```rust
let base = HttpUrl::parse("http://example.com/a/b/c").unwrap();

// Absolute URL
assert_eq!(base.resolve("http://other.com/x").unwrap().host(), "other.com");

// Protocol-relative
assert_eq!(base.resolve("//other.com/x").unwrap().host(), "other.com");

// Absolute path
assert_eq!(base.resolve("/x/y").unwrap().path(), "/x/y");

// Relative path
assert_eq!(base.resolve("d").unwrap().path(), "/a/b/d");
assert_eq!(base.resolve("../d").unwrap().path(), "/a/d");

// Query-only
assert_eq!(base.resolve("?q=1").unwrap().query_parameter("q"), Some("1"));

// Fragment-only
assert_eq!(base.resolve("#sec").unwrap().fragment(), Some("sec"));
```

### `url` Crate Interop

Enable the optional `url` feature:

```toml
[dependencies]
http-url = { version = "0.1", features = ["url"] }
```

```rust
use core::convert::TryFrom;
use http_url::HttpUrl;
use url::Url;

// url::Url → HttpUrl (fallible, only http/https)
let u = Url::parse("https://example.com/path").unwrap();
let http_url = HttpUrl::try_from(u).unwrap();
assert_eq!(http_url.host(), "example.com");

// HttpUrl → url::Url (infallible)
let url: Url = http_url.into();
assert_eq!(url.as_str(), "https://example.com/path");
```

### Feature Flags

| Feature | Default | Description                                               |
|---------|---------|-----------------------------------------------------------|
| `std`   | ✅      | Enables `std::error::Error` impl via `thiserror`          |
| `url`   | ❌      | Enables `From`/`TryFrom` conversions with the `url` crate |

### `no_std`

The crate works without `std`:

```toml
[dependencies]
http-url = { version = "0.1", default-features = false }
```

Only the `alloc` crate is required.

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or [https://www.apache.org/licenses/LICENSE-2.0](https://www.apache.org/licenses/LICENSE-2.0))
- MIT license ([LICENSE-MIT](LICENSE-MIT) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT))

at your option.
