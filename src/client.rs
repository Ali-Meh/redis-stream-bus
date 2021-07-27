use crate::bus::StreamValue;

use super::config::Config;
pub use super::error::{Error, Result};
pub use super::bus::{Stream, StreamBus, StreamID, StreamKey};
use log::*;
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{AsyncCommands, Commands, RedisResult, Value};
use std::collections::HashMap;
use std::usize;
use tokio::sync::mpsc::Receiver;

/// **RedisClient** will keep connection and internal options of redis client
///
///
/// # Fields
/// - `client`: redis-rs client keeping connection and utility
/// - `group_name`: name of group will be joining
/// - `consumer_name`: the name of consumer node
/// - `timeout`: how much block(wait) per event request (ms)
/// - `count`: maximum events per request
///
pub struct RedisClient {
    client: redis::Client,
    connection: redis::Connection,
    group_name: Option<String>,
    consumer_name: Option<String>,
    timeout: usize,
    count: usize,
}

impl RedisClient {
    pub fn new(connection_string: &str) -> RedisResult<Self> {
        let client = redis::Client::open(connection_string)?;
        let connection = client.get_connection()?;
        Ok(RedisClient {
            client,
            connection,
            group_name: None,
            consumer_name: None,
            timeout: 5_000,
            count: 5,
        })
    }

    pub fn with_group_name(mut self, group_name: &str) -> Self {
        self.group_name = Some(group_name.to_owned());
        self
    }

    pub fn with_timeout(mut self, timeout: usize) -> Self {
        self.timeout = timeout;
        self
    }
    
    pub fn with_count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    pub fn with_consumer_name(mut self, consumer_name: &str) -> Self {
        self.consumer_name = Some(consumer_name.to_owned());
        self
    }

    pub fn from_config(config: &Config) -> RedisResult<Self> {
        Ok(Self::new(&config.connection_string)?
            .with_consumer_name(&config.consumer_name)
            .with_group_name(&config.group_name))
    }
}

impl<'a> StreamBus for RedisClient {
    fn ack(&mut self, stream: &Stream) -> Result<()> {
        match &stream.id {
            Some(id) => {
                let res: RedisResult<String> =
                    self.connection.xack(&stream.key, &self.group_name, &[id]);

                match res {
                    Ok(id) => debug!("Stream acknowledged: {:?}", id),
                    Err(err) => error!("An error occurred on acknowledgment: {}", err),
                }
            }
            None => {
                error!("Stream ID is not set for acknowledgment: {:?}", stream);
            }
        }

        Ok(())
    }

    fn add(&mut self, stream: &Stream) -> Result<StreamID> {
        let json = serde_json::to_string(&stream.value).unwrap();
        let id: String = self
            .connection
            .xadd(stream.key.clone(), "*", &[("value", json)])?;

        debug!("Stream added: {:?}", stream);
        Ok(id)
    }

    fn read(&mut self, keys: &Vec<String>) -> Result<Receiver<Stream>> {
        let (read_tx, read_rx) = tokio::sync::mpsc::channel(100);
        let opts =
            StreamReadOptions::default()
            .group(self.group_name.clone(), self.consumer_name.clone())
            .block(self.timeout)
            .count(self.count);

            let mut con = self.client.get_connection()?;
            let mut ids = vec![];
            for k in keys {
                let created: RedisResult<()> =
                    con.xgroup_create_mkstream(k, &self.group_name, "$");
                if let Err(e) = created {
                    println!("Group already exists: {:?} \n", e);
                }
                ids.push(">");
            }
        let keyss=keys.to_owned();

        tokio::spawn(async move {
            loop{
                let stream_option: Option<StreamReadReply> =
                con.xread_options(&keyss, &ids, &opts).unwrap(); // TODO: error handling
                
                match stream_option {
                    Some(reply) => {
                        for key in reply.keys {
                            for id in key.ids {
                                let value = id.map.get("value").unwrap(); // TODO:: Error handling + test
                                match value {
                                    Value::Data(val) => {
                    
                                        let stream = Stream {
                                            id: Some(id.id),
                                            key: key.key.clone(),
                                            value: decode_value(value),
                                        };
                                        println!("[+] Got event {:?}",stream);
                                        read_tx.send(stream).await.unwrap();
                                    }
                                    _ => {
                                        warn!("unimplmented Deserilization {:?}",value);
                                        // TODO
                                    }
                                }
                            }
                        }
                    }
                    None => {
                        debug!("Stream option is empty");
                    }
                }
            }
        });

        Ok(read_rx)
    }
}


fn decode_value(v:&Value) -> StreamValue{
    let map:HashMap<String,String>=redis::from_redis_value(v).unwrap();
    map.into()
}