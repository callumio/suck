use std::sync::{Arc, Mutex};

/// Request messages sent from consumer to producer
pub enum Request {
    GetValue,
    Close,
}

/// Response messages sent from producer to consumer
pub enum Response<T> {
    Value(T),
    NoSource,
    Closed,
}

/// Represents the source of values: either static or dynamic
pub(crate) enum ValueSource<T> {
    Static(T),
    Dynamic(Box<dyn FnMut() -> T + Send + Sync + 'static>),
    None,    // Never set
    Cleared, // Was set but cleared (closed)
}

/// Internal channel state shared between producer and consumer
pub(crate) type ChannelState<T> = Arc<Mutex<ValueSource<T>>>;
