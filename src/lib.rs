#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(feature = "async")]
pub mod async_channel;
pub mod channel;
pub mod error;
pub mod traits;

#[cfg(feature = "async")]
pub mod asynchronous;
#[cfg(feature = "sync")]
pub mod sync;
pub mod types;

#[cfg(feature = "async")]
pub use async_channel::{AsyncSourcer, AsyncSucker};
pub use channel::{Sourcer, Sucker};
pub use error::Error;
