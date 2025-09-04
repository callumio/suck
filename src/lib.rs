#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod channel;
pub mod error;

#[cfg(feature = "sync")]
pub mod sync;
pub mod types;

pub use channel::{Sourcer, Sucker};
pub use error::Error;
