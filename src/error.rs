/// Errors that can occur when using the suck channel
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Channel has been closed gracefully
    ChannelClosed,

    /// Producer has disconnected unexpectedly
    ProducerDisconnected,

    /// No value source has been set
    NoSource,

    /// Closure execution panicked
    ClosurePanic,
}
