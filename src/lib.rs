#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod error;

#[cfg(feature = "async")]
pub mod asynchronous;
#[cfg(feature = "sync")]
pub mod sync;
#[cfg(any(feature = "sync", feature = "async"))]
pub mod types;

#[cfg(feature = "async")]
pub use asynchronous::channel::{AsyncSourcer, AsyncSucker};
#[cfg(feature = "sync")]
pub use sync::channel::{Sourcer, Sucker};
pub use error::Error;
