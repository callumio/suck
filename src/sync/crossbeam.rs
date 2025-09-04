#[cfg(feature = "sync-crossbeam")]
use crate::sync::traits::{ChannelError, ChannelReceiver, ChannelSender, ChannelType};
use crate::types;
use crossbeam_channel;

type CrossbeamSucker<T> =
    crate::Sucker<T, CrossbeamSender<types::Request>, CrossbeamReceiver<types::Response<T>>>;
type CrossbeamSourcer<T> =
    crate::Sourcer<T, CrossbeamReceiver<types::Request>, CrossbeamSender<types::Response<T>>>;

/// Internal sender type for crossbeam backend
pub struct CrossbeamSender<T>(crossbeam_channel::Sender<T>);
/// Internal receiver type for crossbeam backend
pub struct CrossbeamReceiver<T>(crossbeam_channel::Receiver<T>);

impl<T> ChannelSender<T> for CrossbeamSender<T> {
    fn send(&self, msg: T) -> Result<(), ChannelError> {
        self.0
            .send(msg)
            .map_err(|_| ChannelError::ProducerDisconnected)
    }
}

impl<T> ChannelReceiver<T> for CrossbeamReceiver<T> {
    fn recv(&self) -> Result<T, ChannelError> {
        self.0
            .recv()
            .map_err(|_| ChannelError::ProducerDisconnected)
    }
}

/// Internal channel type for crossbeam backend
pub struct CrossbeamChannel;

impl ChannelType for CrossbeamChannel {
    type Sender<T> = CrossbeamSender<T>;
    type Receiver<T> = CrossbeamReceiver<T>;

    fn create_request_channel() -> (Self::Sender<types::Request>, Self::Receiver<types::Request>) {
        let (tx, rx) = crossbeam_channel::unbounded();
        (CrossbeamSender(tx), CrossbeamReceiver(rx))
    }

    fn create_response_channel<T>() -> (
        Self::Sender<types::Response<T>>,
        Self::Receiver<types::Response<T>>,
    ) {
        let (tx, rx) = crossbeam_channel::unbounded();
        (CrossbeamSender(tx), CrossbeamReceiver(rx))
    }
}

pub struct CrossbeamSuck<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> CrossbeamSuck<T> {
    pub fn pair() -> (CrossbeamSucker<T>, CrossbeamSourcer<T>)
    where
        T: Clone + Send + 'static,
    {
        let (request_tx, request_rx) = CrossbeamChannel::create_request_channel();
        let (response_tx, response_rx) = CrossbeamChannel::create_response_channel::<T>();

        let state = std::sync::Arc::new(std::sync::Mutex::new(crate::types::ChannelState {
            source: crate::types::ValueSource::None,
            closed: false,
        }));

        let sucker = crate::Sucker {
            request_tx,
            response_rx,
            closed: std::sync::Mutex::new(false),
            _phantom: std::marker::PhantomData,
        };

        let sourcer = crate::Sourcer {
            request_rx,
            response_tx,
            state: std::sync::Arc::clone(&state),
            _phantom: std::marker::PhantomData,
        };

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
        let (sucker, sourcer) = CrossbeamSuck::<i32>::pair();

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
        let (sucker, sourcer) = CrossbeamSuck::<i32>::pair();

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
        let (sucker, sourcer) = CrossbeamSuck::<i32>::pair();

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
        let (sucker, sourcer) = CrossbeamSuck::<i32>::pair();

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
        let (sucker, sourcer) = CrossbeamSuck::<i32>::pair();

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
        let (sucker, sourcer) = CrossbeamSuck::<i32>::pair();

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
