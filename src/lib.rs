#![doc = include_str!("../README.md")]
pub mod auth;
pub mod oss;
pub mod request;
pub mod url;
pub mod metadata;
mod util;

#[cfg(feature = "blocking")]
pub mod blocking;
#[cfg(not(feature = "blocking"))]
pub mod async_impl;
pub mod entity;
pub mod error;
pub(crate) mod macros;