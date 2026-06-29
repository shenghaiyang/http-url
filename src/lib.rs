#![doc = include_str!("../README.md")]
#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod builder;
mod error;
mod parse;
mod scheme;
mod url;
mod util;

#[cfg(feature = "url")]
mod compat;

pub use builder::HttpUrlBuilder;
pub use error::{HttpUrlError, Result};
pub use scheme::Scheme;
pub use url::HttpUrl;
