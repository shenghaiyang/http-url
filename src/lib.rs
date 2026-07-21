#![doc = include_str!("../README.md")]
#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod builder;
mod error;
mod http_url;
mod scheme;
mod util;

pub use builder::HttpUrlBuilder;
pub use error::{HttpUrlError, Result};
pub use http_url::HttpUrl;
pub use scheme::Scheme;
