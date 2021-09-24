use async_trait::async_trait;
use futures::channel::mpsc::Sender;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[cfg(test)]
use mockall::*;

pub type StreamID = String;

///
/// StreamValue is the data structure passed to event handler
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StreamValue {
    /// identifier of the sender module
    pub module: String,
    // TODO: Do we really need it?
    /// correlation id for the request message
    pub request_id: Option<StreamID>,
    /// request body
    pub message: String,
}

///
/// Stream is the data structure to keep streams nice and tidy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Stream {
    /// redis streams Id auto added via redis node
    pub id: Option<StreamID>,
    /// redis stream this streams belongs to
    pub key: String,
    /// data embedded in the event
    pub value: StreamValue,
}

impl Stream {
    pub fn new<S: Serialize>(
        key: &str,
        module: &str,
        request_id: Option<StreamID>,
        message: S,
    ) -> Self {
        Stream {
            id: None,
            key: key.to_owned(),
            value: StreamValue {
                module: module.to_string(),
                request_id,
                message: serde_json::json!(message).to_string(),
            },
        }
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait StreamBus: Sync + Send {
    fn xadd_sender(&self) -> Sender<Stream>;
    fn xack_sender(&self) -> Sender<Stream>;
    async fn run<'a>(mut self, keys: &[&'a str], mut read_tx: Sender<Stream>);
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
        StreamValue {
            module: parse_to_string(map.get("module")).unwrap_or_default(),
            request_id: parse_to_string(map.get("request_id")),
            message: parse_to_string(map.get("message")).unwrap_or_default(),
        }
    }
}

pub fn parse_to_string(from: Option<&redis::Value>) -> Option<String> {
    if let Some(v) = from {
        match v {
            redis::Value::Data(c) => Some(String::from_utf8(c.clone()).unwrap()),
            redis::Value::Status(c) => Some(c.into()),
            _ => None,
        }
    } else {
        None
    }
}
