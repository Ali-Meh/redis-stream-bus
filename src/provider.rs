use super::error::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{Receiver};

#[cfg(test)]
use mockall::*;

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
    fn ack(&mut self, stream: &Stream) -> Result<()>;
    fn add(&mut self, stream: &Stream) -> Result<StreamID>;
    fn read(&mut self, keys: &Vec<&str>) -> Result<Receiver<Stream>>;
}
