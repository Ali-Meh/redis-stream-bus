#[cfg(test)]
use mockall::*;
use serde::{Deserialize, Serialize};
use async_std::channel::{Receiver, Sender};

pub type StreamID = String;
pub type StreamKey = String;

///
/// EventValue is the data structure passed to event handler
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StreamValue {
    /// identifier of the sender module
    pub module: String,
    // correlation id for the request message
    pub request_id: Option<String>,
    // request body
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Stream {
    pub id: Option<StreamID>,
    pub key: StreamKey,
    pub value: StreamValue,
}
// #[cfg_attr(test, automock)] TODO: how to mock??

pub trait StreamBusClient {
    fn read_receiver(&self) -> Receiver<Stream>;
    fn add_transmitter(&self) -> Sender<Stream>;

    fn run(self);

    // / TODO: this should be a channel
    // /
    // / `ack_event` will notify redis of done processing event and redis can remove it from pending queue
    // /
    // / needs to be run after `process_events` on processed event
    // async fn ack_event(&self, stream_key: StreamKey, stream_id: StreamID);
}
