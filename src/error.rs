/// Errors that can occur when using the suck channel
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Channel has been closed gracefully
    ChannelClosed,

    /// Producer has disconnected unexpectedly
    ProducerDisconnected,

    /// No value source has been set
    NoSource,

    /// Internal error (e.g., mutex poisoning or source execution failure)
    InternalError,
}
