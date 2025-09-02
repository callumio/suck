#![doc = include_str!("../README.md")]

pub mod channel;
pub mod error;
pub mod types;

// Re-exports
pub use channel::{Sourcer, SuckPair, Sucker};
pub use error::Error;
pub use types::ValueSource;

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_pre_computed_value() {
        let (sucker, sourcer) = SuckPair::<i32>::pair();

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
        let (sucker, sourcer) = SuckPair::<i32>::pair();

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
        let (sucker, sourcer) = SuckPair::<i32>::pair();

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
        let (sucker, sourcer) = SuckPair::<i32>::pair();

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
        let (sucker, sourcer) = SuckPair::<i32>::pair();

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
        let (sucker, sourcer) = SuckPair::<i32>::pair();

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
