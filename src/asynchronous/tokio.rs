use std::sync::Arc;

use arc_swap::ArcSwap;
use async_trait::async_trait;
use tokio::sync::{Mutex, mpsc};

use crate::asynchronous::traits::{
    AsyncChannelReceiver, AsyncChannelSender, AsyncChannelType, ChannelError,
};
use crate::types;

type TokioSucker<T> =
    crate::AsyncSucker<T, TokioSender<types::Request>, TokioReceiver<types::Response<T>>>;
type TokioSourcer<T> =
    crate::AsyncSourcer<T, TokioReceiver<types::Request>, TokioSender<types::Response<T>>>;

pub struct TokioSender<T>(mpsc::UnboundedSender<T>);
pub struct TokioReceiver<T>(Mutex<mpsc::UnboundedReceiver<T>>);

#[async_trait]
impl<T: Send + 'static> AsyncChannelSender<T> for TokioSender<T> {
    async fn send(&self, msg: T) -> Result<(), ChannelError> {
        self.0
            .send(msg)
            .map_err(|_| ChannelError::ProducerDisconnected)
    }
}

#[async_trait]
impl<T: Send + 'static> AsyncChannelReceiver<T> for TokioReceiver<T> {
    async fn recv(&self) -> Result<T, ChannelError> {
        let mut receiver = self.0.lock().await;
        receiver
            .recv()
            .await
            .ok_or(ChannelError::ProducerDisconnected)
    }
}

pub struct TokioChannel;

impl AsyncChannelType for TokioChannel {
    type Sender<T: Send + 'static> = TokioSender<T>;
    type Receiver<T: Send + 'static> = TokioReceiver<T>;

    fn create_request_channel() -> (Self::Sender<types::Request>, Self::Receiver<types::Request>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (TokioSender(tx), TokioReceiver(Mutex::new(rx)))
    }

    fn create_response_channel<T: Send + 'static>() -> (
        Self::Sender<types::Response<T>>,
        Self::Receiver<types::Response<T>>,
    ) {
        let (tx, rx) = mpsc::unbounded_channel();
        (TokioSender(tx), TokioReceiver(Mutex::new(rx)))
    }
}

pub struct TokioSuck<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> TokioSuck<T> {
    pub fn pair() -> (TokioSucker<T>, TokioSourcer<T>)
    where
        T: Clone + Send + 'static,
    {
        let (request_tx, request_rx) = TokioChannel::create_request_channel();
        let (response_tx, response_rx) = TokioChannel::create_response_channel::<T>();
        let state = ArcSwap::new(Arc::new(crate::types::ValueSource::None));

        let sucker = crate::AsyncSucker::new(request_tx, response_rx);
        let sourcer = crate::AsyncSourcer::new(request_rx, response_tx, state);

        (sucker, sourcer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;

    #[tokio::test]
    async fn test_pre_computed_value() {
        let (sucker, sourcer) = TokioSuck::<i32>::pair();

        let producer = tokio::spawn(async move {
            sourcer.set_static(42).unwrap();
            sourcer.run().await.unwrap();
        });

        let value = sucker.get().await.unwrap();
        assert_eq!(value, 42);
        sucker.close().await.unwrap();
        producer.await.unwrap();
    }

    #[tokio::test]
    async fn test_no_source_error() {
        let (sucker, sourcer) = TokioSuck::<i32>::pair();

        let producer = tokio::spawn(async move {
            sourcer.run().await.unwrap();
        });

        let result = sucker.get().await;
        assert!(matches!(result, Err(Error::NoSource)));
        sucker.close().await.unwrap();
        producer.await.unwrap();
    }
}
