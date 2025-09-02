use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use crate::error::Error;
use crate::types::{ChannelState, Request, Response, ValueSource};

/// The consumer side of the channel that requests values
pub struct Sucker<T> {
    request_tx: mpsc::Sender<Request>,
    response_rx: mpsc::Receiver<Response<T>>,
    closed: Mutex<bool>,
}

/// The producer side of the channel that provides values
pub struct Sourcer<T> {
    request_rx: mpsc::Receiver<Request>,
    response_tx: mpsc::Sender<Response<T>>,
    state: Arc<Mutex<ChannelState<T>>>,
}

/// Helper type for creating Sucker and Sourcer instances
pub struct SuckPair<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> SuckPair<T> {
    /// Create a new suck pair
    pub fn pair() -> (Sucker<T>, Sourcer<T>)
    where
        T: Clone + Send + 'static,
    {
        let (request_tx, request_rx) = mpsc::channel();
        let (response_tx, response_rx) = mpsc::channel();

        let state = Arc::new(Mutex::new(ChannelState {
            source: ValueSource::None,
            closed: false,
        }));

        let sucker = Sucker {
            request_tx,
            response_rx,
            closed: Mutex::new(false),
        };

        let sourcer = Sourcer {
            request_rx,
            response_tx,
            state: Arc::clone(&state),
        };

        (sucker, sourcer)
    }
}

impl<T> Sourcer<T>
where
    T: Clone + Send + 'static,
{
    /// Set a fixed value
    pub fn set_static(&self, value: T) -> Result<(), Error> {
        let mut state = self.state.lock().map_err(|_| Error::InternalError)?;
        if state.closed {
            return Err(Error::ChannelClosed);
        }
        state.source = ValueSource::Static(value);
        Ok(())
    }

    /// Set a closure
    pub fn set<F>(&self, closure: F) -> Result<(), Error>
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        let mut state = self.state.lock().map_err(|_| Error::InternalError)?;
        if state.closed {
            return Err(Error::ChannelClosed);
        }
        state.source = ValueSource::Dynamic(Box::new(closure));
        Ok(())
    }

    /// Close the channel
    pub fn close(&self) -> Result<(), Error> {
        let mut state = self.state.lock().map_err(|_| Error::InternalError)?;
        state.closed = true;
        state.source = ValueSource::None;
        Ok(())
    }

    /// Handles requests - blocking
    pub fn run(self) -> Result<(), Error> {
        loop {
            match self.request_rx.recv() {
                Ok(Request::GetValue) => {
                    let response = self.handle_get_value()?;
                    if self.response_tx.send(response).is_err() {
                        // Consumer disconnected
                        break;
                    }
                }
                Ok(Request::Close) => {
                    // Close channel
                    let mut state = self.state.lock().map_err(|_| Error::InternalError)?;
                    state.closed = true;
                    state.source = ValueSource::None;
                    break;
                }
                Err(_) => {
                    // Consumer disconnected
                    break;
                }
            }
        }
        Ok(())
    }

    fn handle_get_value(&self) -> Result<Response<T>, Error> {
        let state = self.state.lock().map_err(|_| Error::InternalError)?;
        if state.closed {
            return Ok(Response::Closed);
        }

        match &state.source {
            ValueSource::Static(value) => Ok(Response::Value(value.clone())),
            ValueSource::Dynamic(closure) => {
                let value = self.execute_closure_safely(closure);
                match value {
                    Ok(v) => Ok(Response::Value(v)),
                    Err(_) => Ok(Response::NoSource), // Closure execution failed
                }
            }
            ValueSource::None => Ok(Response::NoSource),
        }
    }

    fn execute_closure_safely(
        &self,
        closure: &dyn Fn() -> T,
    ) -> Result<T, Box<dyn std::any::Any + Send>> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(closure))
    }
}

impl<T> Sucker<T> {
    /// Get the current value from the producer
    pub fn get(&self) -> Result<T, Error> {
        // Check if locally marked as closed
        if *self.closed.lock().unwrap() {
            return Err(Error::ChannelClosed);
        }

        self.request_tx
            .send(Request::GetValue)
            .map_err(|_| Error::ProducerDisconnected)?;

        match self.response_rx.recv() {
            Ok(Response::Value(value)) => Ok(value),
            Ok(Response::NoSource) => Err(Error::NoSource),
            Ok(Response::Closed) => Err(Error::ChannelClosed),
            Err(_) => Err(Error::ProducerDisconnected),
        }
    }

    /// Check if the channel is closed
    pub fn is_closed(&self) -> bool {
        // Send a test request
        self.request_tx.send(Request::GetValue).is_err()
    }

    /// Close the channel from the consumer side
    pub fn close(&self) -> Result<(), Error> {
        // Mark locally as closed
        *self.closed.lock().unwrap() = true;

        // Send close request
        self.request_tx
            .send(Request::Close)
            .map_err(|_| Error::ProducerDisconnected)
    }
}
