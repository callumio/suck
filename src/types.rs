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
    Dynamic(Box<dyn Fn() -> T + Send + Sync + 'static>),
    None,
}

/// Internal channel state shared between producer and consumer
pub(crate) struct ChannelState<T> {
    pub(crate) source: ValueSource<T>,
    pub(crate) closed: bool,
}
