use std::sync::Arc;

#[cfg(feature = "sync-flume")]
use crate::sync::traits::{ChannelError, ChannelReceiver, ChannelSender, ChannelType};
use crate::types;
use arc_swap::ArcSwap;
use flume;

type FlumeSucker<T> =
    crate::Sucker<T, FlumeSender<types::Request>, FlumeReceiver<types::Response<T>>>;
type FlumeSourcer<T> =
    crate::Sourcer<T, FlumeReceiver<types::Request>, FlumeSender<types::Response<T>>>;

/// Internal sender type for flume backend  
pub struct FlumeSender<T>(flume::Sender<T>);
/// Internal receiver type for flume backend
pub struct FlumeReceiver<T>(flume::Receiver<T>);

impl<T> ChannelSender<T> for FlumeSender<T> {
    fn send(&self, msg: T) -> Result<(), ChannelError> {
        self.0
            .send(msg)
            .map_err(|_| ChannelError::ProducerDisconnected)
    }
}

impl<T> ChannelReceiver<T> for FlumeReceiver<T> {
    fn recv(&self) -> Result<T, ChannelError> {
        self.0
            .recv()
            .map_err(|_| ChannelError::ProducerDisconnected)
    }
}

/// Internal channel type for flume backend
pub struct FlumeChannel;

impl ChannelType for FlumeChannel {
    type Sender<T> = FlumeSender<T>;
    type Receiver<T> = FlumeReceiver<T>;

    fn create_request_channel() -> (Self::Sender<types::Request>, Self::Receiver<types::Request>) {
        let (tx, rx) = flume::unbounded();
        (FlumeSender(tx), FlumeReceiver(rx))
    }

    fn create_response_channel<T>() -> (
        Self::Sender<types::Response<T>>,
        Self::Receiver<types::Response<T>>,
    ) {
        let (tx, rx) = flume::unbounded();
        (FlumeSender(tx), FlumeReceiver(rx))
    }
}

pub struct FlumeSuck<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> FlumeSuck<T> {
    pub fn pair() -> (FlumeSucker<T>, FlumeSourcer<T>)
    where
        T: Clone + Send + 'static,
    {
        let (request_tx, request_rx) = FlumeChannel::create_request_channel();
        let (response_tx, response_rx) = FlumeChannel::create_response_channel::<T>();

        let state = Arc::new(crate::types::ValueSource::None);
        let state = ArcSwap::new(state);

        let sucker = crate::Sucker::new(request_tx, response_rx);
        let sourcer = crate::Sourcer::new(request_rx, response_tx, state);

        (sucker, sourcer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;
    use std::thread;

    #[test]
    fn test_pre_computed_value() {
        let (sucker, sourcer) = FlumeSuck::<i32>::pair();

        // Start producer
        let producer_handle = thread::spawn(move || {
            sourcer.set_static(42).unwrap();
            sourcer.run().unwrap();
        });

        // Ensure consumer gets the value
        let value = sucker.get().unwrap();
        assert_eq!(value, 42);

        // Close consumer
        sucker.close().unwrap();

        producer_handle.join().unwrap();
    }

    #[test]
    fn test_closure_value() {
        let (sucker, sourcer) = FlumeSuck::<i32>::pair();

        // Start producer
        let producer_handle = std::thread::spawn(move || {
            let counter = std::sync::Arc::new(std::sync::Mutex::new(0));
            let counter_clone = std::sync::Arc::clone(&counter);
            sourcer
                .set(move || {
                    let mut count = counter_clone.lock().unwrap();
                    *count += 1;
                    *count
                })
                .unwrap();
            sourcer.run().unwrap();
        });

        // Ensure consumer gets the value
        let value1 = sucker.get().unwrap();
        assert_eq!(value1, 1);

        // Ensure consumer gets the next value
        let value2 = sucker.get().unwrap();
        assert_eq!(value2, 2);

        // Close consumer
        sucker.close().unwrap();

        producer_handle.join().unwrap();
    }

    #[test]
    fn test_no_source_error() {
        let (sucker, sourcer) = FlumeSuck::<i32>::pair();

        // Start producer
        let producer_handle = thread::spawn(move || {
            sourcer.run().unwrap();
        });

        // Consumer should get NoSource error
        let result = sucker.get();
        assert!(matches!(result, Err(Error::NoSource)));

        // Close consumer
        sucker.close().unwrap();

        producer_handle.join().unwrap();
    }

    #[test]
    fn test_channel_closed_error() {
        let (sucker, sourcer) = FlumeSuck::<i32>::pair();

        // Start producer
        let producer_handle = thread::spawn(move || {
            sourcer.set_static(42).unwrap();
            sourcer.run().unwrap();
        });

        // Close consumer
        sucker.close().unwrap();

        let result = sucker.get();
        assert!(matches!(result, Err(Error::ChannelClosed)));

        producer_handle.join().unwrap();
    }

    #[test]
    fn test_producer_disconnection_error() {
        let (sucker, sourcer) = FlumeSuck::<i32>::pair();

        // Start producer
        let producer_handle = thread::spawn(move || {
            sourcer.set_static(42).unwrap();
            // Simulate crash
            panic!("Producer crashed!");
        });

        let result = sucker.get();
        assert!(matches!(result, Err(Error::ProducerDisconnected)));

        let _ = producer_handle.join();
    }

    #[test]
    fn test_is_closed() {
        let (sucker, sourcer) = FlumeSuck::<i32>::pair();

        assert!(!sucker.is_closed());

        // Start producer
        let producer_handle = thread::spawn(move || {
            sourcer.set_static(42).unwrap();
            sourcer.run().unwrap();
        });

        // Get one value
        let _ = sucker.get().unwrap();
        assert!(!sucker.is_closed());

        // Close and check
        sucker.close().unwrap();

        producer_handle.join().unwrap();
    }
}
