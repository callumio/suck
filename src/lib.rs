#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(feature = "async")]
pub mod async_channel;
#[cfg(feature = "sync")]
pub mod channel;
pub mod error;
#[cfg(feature = "sync")]
pub mod traits;

#[cfg(feature = "async")]
pub mod asynchronous;
#[cfg(feature = "sync")]
pub mod sync;
#[cfg(any(feature = "sync", feature = "async"))]
pub mod types;

#[cfg(feature = "async")]
pub use async_channel::{AsyncSourcer, AsyncSucker};
#[cfg(feature = "sync")]
pub use channel::{Sourcer, Sucker};
pub use error::Error;
