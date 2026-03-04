use async_trait::async_trait;

pub use crate::error::Error as ChannelError;

#[async_trait]
pub trait AsyncChannelSender<T>: Send + Sync {
    async fn send(&self, msg: T) -> Result<(), ChannelError>;
}

#[async_trait]
pub trait AsyncChannelReceiver<T>: Send + Sync {
    async fn recv(&self) -> Result<T, ChannelError>;
}

pub trait AsyncChannelType {
    type Sender<T: Send + 'static>: AsyncChannelSender<T>;
    type Receiver<T: Send + 'static>: AsyncChannelReceiver<T>;

    fn create_request_channel() -> (
        Self::Sender<crate::types::Request>,
        Self::Receiver<crate::types::Request>,
    );
    fn create_response_channel<T: Send + 'static>() -> (
        Self::Sender<crate::types::Response<T>>,
        Self::Receiver<crate::types::Response<T>>,
    );
}
