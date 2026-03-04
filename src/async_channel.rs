use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::asynchronous::traits::{AsyncChannelReceiver, AsyncChannelSender};
use crate::error::Error;
use crate::types::{ChannelState, Request, Response, ValueSource};

/// The consumer side of the channel that requests values asynchronously.
pub struct AsyncSucker<T, ST, SR>
where
    ST: AsyncChannelSender<Request>,
    SR: AsyncChannelReceiver<Response<T>>,
{
    request_tx: ST,
    response_rx: SR,
    closed: AtomicBool,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, ST, SR> AsyncSucker<T, ST, SR>
where
    ST: AsyncChannelSender<Request>,
    SR: AsyncChannelReceiver<Response<T>>,
{
    pub(crate) fn new(request_tx: ST, response_rx: SR) -> Self {
        Self {
            request_tx,
            response_rx,
            closed: AtomicBool::new(false),
            _phantom: std::marker::PhantomData,
        }
    }
}

/// The producer side of the channel that provides values asynchronously.
pub struct AsyncSourcer<T, SR, ST>
where
    SR: AsyncChannelReceiver<Request>,
    ST: AsyncChannelSender<Response<T>>,
{
    request_rx: SR,
    response_tx: ST,
    state: ChannelState<T>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, SR, ST> AsyncSourcer<T, SR, ST>
where
    SR: AsyncChannelReceiver<Request>,
    ST: AsyncChannelSender<Response<T>>,
{
    pub(crate) fn new(request_rx: SR, response_tx: ST, state: ChannelState<T>) -> Self {
        Self {
            request_rx,
            response_tx,
            state,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, SR, ST> AsyncSourcer<T, SR, ST>
where
    T: Send + 'static,
    SR: AsyncChannelReceiver<Request>,
    ST: AsyncChannelSender<Response<T>>,
{
    pub fn set_static(&self, val: T) -> Result<(), Error>
    where
        T: Clone,
    {
        self.state.swap(Arc::new(ValueSource::Static {
            val,
            clone: T::clone,
        }));
        Ok(())
    }

    pub fn set<F>(&self, closure: F) -> Result<(), Error>
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.state
            .swap(Arc::new(ValueSource::Dynamic(Box::new(closure))));
        Ok(())
    }

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

    pub fn close(&self) -> Result<(), Error> {
        self.state.swap(Arc::new(ValueSource::Cleared));
        Ok(())
    }

    pub async fn run(self) -> Result<(), Error> {
        loop {
            match self.request_rx.recv().await {
                Ok(Request::GetValue) => {
                    let response = self.handle_get_value()?;
                    if self.response_tx.send(response).await.is_err() {
                        break;
                    }
                }
                Ok(Request::Close) => {
                    self.close()?;
                    break;
                }
                Err(_) => break,
            }
        }
        Ok(())
    }

    fn handle_get_value(&self) -> Result<Response<T>, Error> {
        let state = self.state.load();

        match &**state {
            ValueSource::Static { val, clone } => {
                let value = self.execute_closure_safely(&mut || clone(val));
                match value {
                    Ok(v) => Ok(Response::Value(v)),
                    Err(_) => Ok(Response::NoSource),
                }
            }
            ValueSource::Dynamic(closure) => {
                let value = self.execute_closure_safely(&mut || closure());
                match value {
                    Ok(v) => Ok(Response::Value(v)),
                    Err(_) => Ok(Response::NoSource),
                }
            }
            ValueSource::DynamicMut(closure) => {
                let mut closure = closure.lock().unwrap();
                let value = self.execute_closure_safely(&mut *closure);
                match value {
                    Ok(v) => Ok(Response::Value(v)),
                    Err(_) => Ok(Response::NoSource),
                }
            }
            ValueSource::None => Ok(Response::NoSource),
            ValueSource::Cleared => Ok(Response::Closed),
        }
    }

    fn execute_closure_safely(
        &self,
        closure: &mut dyn FnMut() -> T,
    ) -> Result<T, Box<dyn std::any::Any + Send>> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(closure))
    }
}

impl<T, ST, SR> AsyncSucker<T, ST, SR>
where
    ST: AsyncChannelSender<Request>,
    SR: AsyncChannelReceiver<Response<T>>,
{
    pub async fn get(&self) -> Result<T, Error> {
        if self.closed.load(Ordering::Acquire) {
            return Err(Error::ChannelClosed);
        }

        self.request_tx
            .send(Request::GetValue)
            .await
            .map_err(|_| Error::ProducerDisconnected)?;

        match self.response_rx.recv().await {
            Ok(Response::Value(value)) => Ok(value),
            Ok(Response::NoSource) => Err(Error::NoSource),
            Ok(Response::Closed) => Err(Error::ChannelClosed),
            Err(_) => Err(Error::ProducerDisconnected),
        }
    }

    pub async fn is_closed(&self) -> bool {
        self.request_tx.send(Request::GetValue).await.is_err()
    }

    pub async fn close(&self) -> Result<(), Error> {
        self.closed.store(true, Ordering::Release);
        self.request_tx
            .send(Request::Close)
            .await
            .map_err(|_| Error::ProducerDisconnected)
    }
}
