use std::sync::Mutex;

use arc_swap::ArcSwap;

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
    Static { val: T, clone: fn(&T) -> T },
    DynamicMut(Mutex<Box<dyn FnMut() -> T + Send + Sync + 'static>>),
    Dynamic(Box<dyn Fn() -> T + Send + Sync + 'static>),
    None,    // Never set
    Cleared, // Was set but cleared (closed)
}

/// Internal channel state shared between producer and consumer
pub(crate) type ChannelState<T> = ArcSwap<ValueSource<T>>;
