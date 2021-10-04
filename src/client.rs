use super::config::Config;
pub use super::{bus::StreamBus, stream::Stream};
use async_trait::async_trait;
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::FutureExt;
use futures::{select, SinkExt};
use futures_util::StreamExt;
#[cfg(not(test))]
use log::{error, info, trace, warn};
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::{AsyncCommands, RedisResult};
use std::collections::BTreeMap;
use std::pin::Pin;
use std::usize;

#[cfg(test)]
use std::{println as trace, println as info, println as warn, println as error};

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
    group_name: String,
    consumer_name: String,
    timeout: usize,
    count: usize,
    add_ch: (Sender<Stream>, Receiver<Stream>),
    ack_ch: (Sender<Stream>, Receiver<Stream>),
}

impl RedisClient {
    pub fn new(
        connection_string: &str,
        group_name: &str,
        consumer_name: &str,
    ) -> RedisResult<Self> {
        let client = redis::Client::open(connection_string)?;
        Ok(RedisClient {
            client,
            group_name: group_name.to_owned(),
            consumer_name: consumer_name.to_owned(),
            timeout: 5_000,
            count: 5,
            add_ch: channel(100),
            ack_ch: channel(100),
        })
    }

    pub fn with_timeout(mut self, timeout: usize) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    pub fn from_config(config: &Config) -> RedisResult<Self> {
        Self::new(
            &config.connection_string,
            &config.group_name,
            &config.consumer_name,
        )
    }
}

#[async_trait]
impl StreamBus for RedisClient {
    fn xadd_sender(&self) -> Sender<Stream> {
        self.add_ch.0.clone()
    }
    fn xack_sender(&self) -> Sender<Stream> {
        self.ack_ch.0.clone()
    }

    async fn run<'a>(mut self, keys: &[&'a str], mut read_tx: Sender<Stream>) {
        let mut con_read = self.client.get_tokio_connection().await.unwrap();
        let mut con_add = self.client.get_tokio_connection().await.unwrap();
        let mut con_ack = self.client.get_tokio_connection().await.unwrap();

        let opts = StreamReadOptions::default()
            .group(self.group_name.clone(), self.consumer_name.clone())
            .block(self.timeout)
            .count(self.count);

        let mut ids = vec![];
        for k in keys {
            let created: RedisResult<()> = con_read
                .xgroup_create_mkstream(k, &self.group_name, "$")
                .await;

            match created {
                Ok(_) => trace!("Group created successfully: {}", self.group_name),
                Err(err) => warn!(
                    "An error occurred when creating a group {:?}: {:?}",
                    self.group_name, err
                ),
            }
            ids.push(">");
        }

        info!("Started listening on events. on keys: {:?}", keys);

        loop {
            let mut read_stream: futures_util::future::IntoStream<
                Pin<
                    Box<
                        dyn futures_util::Future<
                                Output = std::result::Result<StreamReadReply, redis::RedisError>,
                            > + std::marker::Send,
                    >,
                >,
            > = con_read.xread_options(keys, &ids, &opts).into_stream();

            select! {
                read_option = read_stream.next() => if let Some(stream_result) = read_option {
                    match stream_result {
                        Ok(stream) =>{
                            for stream_key in stream.keys {
                                for stream_id in stream_key.ids {
                                    let stream = Stream::new(&stream_key.key, Some(stream_id.id), stream_id.map);
                                    trace!("[>][{}]", &stream.key);
                                    if let Err(err) = read_tx.send(stream).await {
                                        error!("[!]: {:?}", err);
                                        break;
                                    };
                                }
                            }
                        },
                        Err (err) => {
                            error!("[!]: {}", err);
                            break;
                        }
                    }
                },
                add_option = self.add_ch.1.next() => if let Some(stream) = add_option {
                    let stream_id = match stream.id {
                        Some(id) => id,
                        None => "*".to_owned(),
                    };

                    let mut map = BTreeMap::new();
                    for (k, v) in stream.fields {
                        match v {
                            redis::Value::Data(d) => {
                                map.insert(k, d);
                            }
                            _ => {}
                        }
                    }
                    match con_add.xadd_map::<_, _, BTreeMap<String, Vec<u8>>, String>(
                        stream.key.clone(),
                        stream_id,
                        map,
                    ).await {
                        Ok(id) => {
                            trace!("[<][{}]: {}", &stream.key, &id);
                        }
                        Err(e)=> {
                            error!("[!][{}]: {}", &stream.key, e);
                            break;
                        }
                    }
                },
                ack_option = self.ack_ch.1.next() => if let Some(stream) = ack_option {
                    match stream.id {
                        Some(id) => {
                            trace!("[^][{}]: {}", &stream.key, &id);
                            if let Err(err) = con_ack.xack::<_, _, _, i32>(&stream.key, &self.group_name, &[id]).await{
                                error!("[!][{}]: {}", &stream.key, err);
                            }
                        }
                        None => {
                            error!("[!][{}]: Stream ID is not set for the acknowledgment: {:?}", &stream.key, stream);
                            break;
                        }
                    }
                },
            }
        }
    }
}
