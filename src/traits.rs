pub use crate::error::Error as ChannelError;

pub trait ChannelSender<T> {
    fn send(&self, msg: T) -> Result<(), ChannelError>;
}

pub trait ChannelReceiver<T> {
    fn recv(&self) -> Result<T, ChannelError>;
}

pub trait ChannelType {
    type Sender<T>: ChannelSender<T>;
    type Receiver<T>: ChannelReceiver<T>;

    fn create_request_channel() -> (
        Self::Sender<crate::types::Request>,
        Self::Receiver<crate::types::Request>,
    );
    fn create_response_channel<T>() -> (
        Self::Sender<crate::types::Response<T>>,
        Self::Receiver<crate::types::Response<T>>,
    );
}
