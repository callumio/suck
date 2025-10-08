use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::error::Error;
use crate::sync::traits::{ChannelReceiver, ChannelSender};
use crate::types::{ChannelState, Request, Response, ValueSource};

/// The consumer side of the channel that requests values
pub struct Sucker<T, ST, SR>
where
    ST: ChannelSender<Request>,
    SR: ChannelReceiver<Response<T>>,
{
    request_tx: ST,
    response_rx: SR,
    closed: AtomicBool,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, ST, SR> Sucker<T, ST, SR>
where
    ST: ChannelSender<Request>,
    SR: ChannelReceiver<Response<T>>,
{
    /// Create a new Sucker instance
    pub(crate) fn new(request_tx: ST, response_rx: SR) -> Self {
        Self {
            request_tx,
            response_rx,
            closed: AtomicBool::new(false),
            _phantom: std::marker::PhantomData,
        }
    }
}

/// The producer side of the channel that provides values
pub struct Sourcer<T, SR, ST>
where
    SR: ChannelReceiver<Request>,
    ST: ChannelSender<Response<T>>,
{
    request_rx: SR,
    response_tx: ST,
    state: ChannelState<T>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, SR, ST> Sourcer<T, SR, ST>
where
    SR: ChannelReceiver<Request>,
    ST: ChannelSender<Response<T>>,
{
    /// Create a new Sourcer instance
    pub(crate) fn new(request_rx: SR, response_tx: ST, state: ChannelState<T>) -> Self {
        Self {
            request_rx,
            response_tx,
            state,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, SR, ST> Sourcer<T, SR, ST>
where
    T: Clone + Send + 'static,
    SR: ChannelReceiver<Request>,
    ST: ChannelSender<Response<T>>,
{
    /// Set a fixed value
    pub fn set_static(&self, value: T) -> Result<(), Error> {
        self.state.swap(Arc::new(ValueSource::Static(value)));
        Ok(())
    }

    /// Set a closure that implements [Fn]
    pub fn set<F>(&self, closure: F) -> Result<(), Error>
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.state
            .swap(Arc::new(ValueSource::Dynamic(Box::new(closure))));
        Ok(())
    }

    /// Set a closure that implements [FnMut]
    pub fn set_mut<F>(&self, closure: F) -> Result<(), Error>
    where
        F: FnMut() -> T + Send + Sync + 'static,
    {
        self.state
            .swap(Arc::new(ValueSource::DynamicMut(Mutex::new(Box::new(
                closure,
            )))));
        Ok(())
    }

    /// Close the channel
    pub fn close(&self) -> Result<(), Error> {
        self.state.swap(Arc::new(ValueSource::Cleared));
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
                    self.close()?;
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
        let state = self.state.load();

        match &**state {
            ValueSource::Static(value) => Ok(Response::Value(value.clone())),
            ValueSource::Dynamic(closure) => {
                let value = self.execute_closure_safely(&mut || closure());
                match value {
                    Ok(v) => Ok(Response::Value(v)),
                    Err(_) => Ok(Response::NoSource), // Closure execution failed
                }
            }
            ValueSource::DynamicMut(closure) => {
                let mut closure = closure.lock().unwrap();
                let value = self.execute_closure_safely(&mut *closure);
                match value {
                    Ok(v) => Ok(Response::Value(v)),
                    Err(_) => Ok(Response::NoSource), // Closure execution failed
                }
            }
            ValueSource::None => Ok(Response::NoSource), // No source was ever set
            ValueSource::Cleared => Ok(Response::Closed), // Channel was closed (source was set then cleared)
        }
    }

    fn execute_closure_safely(
        &self,
        closure: &mut dyn FnMut() -> T,
    ) -> Result<T, Box<dyn std::any::Any + Send>> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(closure))
    }
}

impl<T, ST, SR> Sucker<T, ST, SR>
where
    ST: ChannelSender<Request>,
    SR: ChannelReceiver<Response<T>>,
{
    /// Get the current value from the producer
    pub fn get(&self) -> Result<T, Error> {
        // Check if locally marked as closed
        if self.closed.load(Ordering::Acquire) {
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
        self.closed.store(true, Ordering::Release);

        // Send close request
        self.request_tx
            .send(Request::Close)
            .map_err(|_| Error::ProducerDisconnected)
    }
}
