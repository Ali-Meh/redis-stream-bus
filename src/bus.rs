use std::collections::{BTreeMap, HashMap};

use super::error::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Receiver;

#[cfg(test)]
use mockall::*;

pub type StreamID = String;
pub type StreamKey = String;

///
/// EventValue is the data structure passed to event handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamValue {
    /// identifier of the sender module
    pub module: String,
    // correlation id for the request message
    pub request_id: Option<String>,
    // request body
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stream {
    pub id: Option<StreamID>,
    pub key: StreamKey,
    pub value: StreamValue,
}
#[cfg_attr(test, automock)]
pub trait StreamBus: Sync + Send + 'static {
    fn ack(&mut self, stream: &Stream) -> Result<()>;
    fn add(&mut self, stream: &Stream) -> Result<StreamID>;
    fn read(&mut self, keys: &Vec<String>) -> Result<Receiver<Stream>>;
}

impl From<StreamValue> for BTreeMap<String, String> {
    fn from(event: StreamValue) -> Self {
        let mut map: BTreeMap<String, String> = BTreeMap::new();

        map.insert("module".to_string(), event.module);
        map.insert("message".to_string(), event.message);

        if let Some(rid) = event.request_id {
            map.insert("request_id".to_string(), rid);
        }

        map
    }
}

impl From<HashMap<String, redis::Value>> for StreamValue {
    fn from(map: HashMap<String, redis::Value>) -> Self {
        let request_id = parse_to_string(map.get("request_id"));
        let request_id = if "".to_string() == request_id {
            None
        } else {
            Some(request_id)
        };

        StreamValue {
            module: parse_to_string(map.get("module")),
            request_id,
            message: parse_to_string(map.get("message")),
        }
    }
}

pub fn parse_to_string(from: Option<&redis::Value>) -> String {
    if let Some(v) = from {
        match v {
            redis::Value::Data(c) => String::from_utf8(c.clone()).unwrap(),
            redis::Value::Status(c) => c.into(),
            // Value::Int(c)=>c.into(),
            _ => String::from(""),
        }
    } else {
        String::from("")
    }
}
