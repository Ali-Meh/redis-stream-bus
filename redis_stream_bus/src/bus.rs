use async_trait::async_trait;
use futures::channel::mpsc::Sender;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[cfg(test)]
use mockall::*;

pub type StreamID = String;


/// Stream is the data structure to keep streams nice and tidy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Stream {
    /// redis streams Id auto added via redis node
    pub id: Option<StreamID>,
    /// redis stream this streams belongs to
    pub key: String,
    /// data embedded in the event
    pub message: Message,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message{
    pub fields : serde_json::Value
}

impl Stream{
    pub fn new(key:&str, fields:serde_json::Value)->Self{
        Stream{
            id:None,
            key:key.to_string(),
            message:Message{
                fields
            }
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

impl From<Message> for BTreeMap<String, String> {
    fn from(msg: Message) -> Self {
        let mut map: BTreeMap<String, String> = BTreeMap::new();
        map.append(&mut serde_json::from_value(msg.fields).unwrap());
        map
    }
}

impl From<HashMap<String, redis::Value>> for Message {
    fn from(map: HashMap<String, redis::Value>) -> Self {
        let mut serde_map:HashMap<String,serde_json::Value>=HashMap::new();
        for kv in map.clone().into_iter(){
            serde_map.insert(kv.0, serde_json::Value::String(parse_to_string(Some(&kv.1))));
        }

        let fields=serde_json::to_value(serde_map).unwrap();

        Message {
            fields
        }
    }
}

pub fn parse_to_string(from: Option<&redis::Value>) -> String {
    if let Some(v) = from {
        match v {
            redis::Value::Data(c) => String::from_utf8(c.clone()).unwrap(),
            redis::Value::Status(c) => c.into(),
            _ => String::from(""),
        }
    } else {
        String::from("")
    }
}
