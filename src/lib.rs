#![doc = include_str!("../README.md")]

pub mod channel;
pub mod error;
pub mod types;

// Re-exports
pub use channel::{Sourcer, SuckPair, Sucker};
pub use error::Error;
pub use types::ValueSource;
