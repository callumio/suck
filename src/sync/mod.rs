pub mod traits;

#[cfg(feature = "sync-crossbeam")]
pub mod crossbeam;
#[cfg(feature = "sync-flume")]
pub mod flume;
#[cfg(feature = "sync-std")]
pub mod std;

#[cfg(feature = "sync-flume")]
pub use flume::FlumeSuck;

#[cfg(feature = "sync-crossbeam")]
pub use crossbeam::CrossbeamSuck;

#[cfg(feature = "sync-std")]
pub use std::StdSuck;
