# http-url

[![Crates.io](https://img.shields.io/crates/v/http-url.svg)](https://crates.io/crates/http-url)
[![Documentation](https://docs.rs/http-url/badge.svg)](https://docs.rs/http-url)
[![CI](https://github.com/shenghaiyang/http-url/actions/workflows/ci.yml/badge.svg)](https://github.com/shenghaiyang/http-url/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE-APACHE)

A focused **builder** for HTTP/HTTPS URLs, powered by the [`url`] crate.

`http-url` does **not** reimplement URL parsing. Parsing, percent-encoding,
IDNA normalization and relative-reference resolution are all delegated to
[`url`] (the WHATWG URL Standard implementation). What this crate adds on top
is:

- a fluent `HttpUrlBuilder` for assembling URLs programmatically,
- an `HttpUrl` value type that enforces the "scheme is `http` or `https`"
  invariant, with a typed `Scheme` accessor.

Component access (host, path, query, fragment, …) is provided directly by
[`url::Url`] via `HttpUrl::as_url`; this crate does not re-wrap those
getters.

[`url`]: https://crates.io/crates/url
[`url::Url`]: https://docs.rs/url/latest/url/struct.Url.html

## Features

- Fluent HTTP/HTTPS URL builder
- `HttpUrl` as a typed, scheme-restricted newtype over [`url::Url`]
- `no_std` + `alloc` support

## Quick Start

```toml
[dependencies]
http-url = "0.2"
```

```rust
use http_url::{HttpUrl, Scheme};

// Build
let http_url = HttpUrl::builder()
    .scheme(Scheme::Https)
    .host("api.example.com")
    .add_path_segment("v1")
    .add_path_segment("users")
    .add_query_parameter("id", "42")
    .build()
    .unwrap();

assert_eq!(http_url.to_string(), "https://api.example.com/v1/users?id=42");

// Parse (delegated to the `url` crate). Only http/https are accepted.
let http_url = HttpUrl::parse("https://user:pass@example.com:8080/path/to?a=1&b=2#frag").unwrap();

assert_eq!(http_url.scheme(), "https");

// Reach the full url::Url API for component access.
let url = http_url.as_url();
assert_eq!(url.username(), "user");
assert_eq!(url.password(), Some("pass"));
assert_eq!(url.host_str(), Some("example.com"));
assert_eq!(url.port(), Some(8080));
assert_eq!(url.path(), "/path/to");
assert_eq!(url.query(), Some("a=1&b=2"));
assert_eq!(url.fragment(), Some("frag"));
```

### Builder API

```rust,ignore
HttpUrl::builder()
    .scheme(Scheme::Https)              // Scheme::Http | Scheme::Https
    .username("user")                   // optional
    .password("pass")                   // optional
    .host("example.com")                // domain, IPv4, or [IPv6]
    .port(8080)                         // optional, uses scheme default otherwise
    .remove_port()                      // revert to scheme default
    .add_path_segment("api")            // decoded segment (auto-encoded)
    .add_path_segment("v1")
    .add_path_segments(&["a", "b"])     // multiple segments at once
    .set_path("/a/b/c")                 // replace entire path (decoded)
    .add_query_parameter("key", "val")  // decoded form
    .remove_query_parameter("key")      // remove all with this name
    .clear_query_parameters()           // remove all
    .fragment("section")                // decoded fragment
    .remove_fragment()                  // clear fragment
    .build_url()                        // -> url::Url
    .unwrap();
// or .build() -> HttpUrl
```

All components are stored in **decoded** form on the builder and handed to
[`url::Url`] on `build()` / `build_url()`, which performs percent-encoding and
normalization.

### Modifying an existing URL

A builder can be started from a parsed URL — a string, an [`url::Url`], or an
existing `HttpUrl` — and then modified:

```rust
use http_url::{HttpUrl, HttpUrlBuilder};

// From a string
let url = HttpUrlBuilder::parse("https://example.com/a/b?x=1")
    .unwrap()
    .add_path_segment("c")
    .add_query_parameter("y", "2")
    .build()
    .unwrap();
assert_eq!(url.to_string(), "https://example.com/a/b/c?x=1&y=2");

// From an existing HttpUrl
let url = HttpUrl::parse("https://example.com/a/b?x=1").unwrap();
let url2 = url.new_builder()
    .add_path_segment("c")
    .build()
    .unwrap();
assert_eq!(url2.to_string(), "https://example.com/a/b/c?x=1");
```

### `url` Crate Interop

`url` is a required dependency, and `HttpUrl` is a thin newtype over
[`url::Url`]. Use the conversion methods to move between the two:

```rust
use http_url::HttpUrl;

// url::Url → HttpUrl (fallible, only http/https)
let u = url::Url::parse("https://example.com/path").unwrap();
let http_url = HttpUrl::try_from(u.clone()).unwrap();
assert_eq!(http_url.as_url(), &u);

// HttpUrl → url::Url (infallible, move)
let url: url::Url = http_url.into();
assert_eq!(url, u);
```

### Feature Flags

| Feature | Default | Description                                      |
|---------|---------|--------------------------------------------------|
| `std`   | ✅      | Enables `std::error::Error` impl via `thiserror` |

### `no_std`

The crate works without `std`, requiring only `alloc`:

```toml
[dependencies]
http-url = { version = "0.2", default-features = false }
```

The underlying [`url`] crate is itself `no_std` + `alloc` compatible.

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <https://opensource.org/licenses/MIT>)

at your option.
