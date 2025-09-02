use thiserror::Error;

/// Errors that can occur when using the suck channel
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Channel has been closed gracefully
    #[error("Channel closed")]
    ChannelClosed,

    /// Producer has disconnected unexpectedly
    #[error("Producer disconnected")]
    ProducerDisconnected,

    /// No value source has been set
    #[error("Producer has not set a source value")]
    NoSource,

    /// Internal error (e.g., mutex poisoning or source execution failure)
    #[error("Internal error occurred")] // TODO: Expand on this
    InternalError,
}
