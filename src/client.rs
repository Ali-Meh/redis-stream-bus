pub use super::provider::{Stream, StreamBusClient, StreamID, StreamKey};
use async_std::channel::{Receiver, Sender};
use async_trait::async_trait;
use futures::{future::FutureExt, select};
use futures_util::stream::StreamExt;
use log::*;
use redis::streams::{StreamReadOptions};
use redis::{AsyncCommands, RedisFuture, RedisResult};
use std::collections::{BTreeMap};
use std::{ usize};

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
    keys: Vec<StreamKey>,
    add_ch: (Sender<Stream>, Receiver<Stream>),
    read_ch: (Sender<Stream>, Receiver<Stream>),
    client: redis::Client,
    group_name: Option<String>,
    consumer_name: Option<String>,
    timeout: usize,
    count: usize,
}

impl RedisClient {
    pub fn new(connection_string: &str) -> Self {
        let client = redis::Client::open(connection_string).unwrap();
        RedisClient {
            keys: Vec::new(),
            add_ch: async_std::channel::unbounded(),
            read_ch: async_std::channel::unbounded(),
            client: client,
            group_name: None,
            consumer_name: None,
            timeout: 5_000,
            count: 5,
        }
    }

    pub fn with_group_name(mut self, group_name: &str) -> Self {
        self.group_name = Some(group_name.to_owned());
        self
    }

    pub fn with_consumer_name(mut self, consumer_name: &str) -> Self {
        self.consumer_name = Some(consumer_name.to_owned());
        self
    }
}

#[async_trait]
impl StreamBusClient for RedisClient {
    fn read_receiver(&self) -> Receiver<Stream> {
        self.read_ch.1.clone()
    }
    fn add_transmitter(&self) -> Sender<Stream> {
        self.add_ch.0.clone()
    }

    fn run(mut self) {
        tokio::spawn(async move {
            let mut ids = vec![];
            for k in self.keys {
                //     let created: RedisResult<()> =
                //         con.xgroup_create_mkstream(*k, &self.group_name, id).await;
                //     if let Err(e) = created {
                //         debug!("Group already exists: {:?} \n", e);
                //     }
                ids.push(">");
            }

            loop {
                let mut con1 = self.client.get_tokio_connection().await.unwrap();
                let mut con2 = self.client.get_tokio_connection().await.unwrap();

                let opts = StreamReadOptions::default()
                    .block(self.timeout)
                    .count(self.count)
                    .group(&self.group_name, &self.consumer_name);

                let xread_fut: RedisFuture<String> = con1.xread_options(&[&""], &ids, &opts);
                let mut read_fuse = xread_fut.fuse();

                select! {
                    a = read_fuse => {}

                            read_data = self.add_ch.1.next() => match read_data{
                        Some(data) => {
                            println!("data {:?}", data);
                            let map:BTreeMap<String,String>=BTreeMap::new();
                            let id: RedisResult<String> = con2.xadd_map(
                                0, // maxlen,
                                "*",
                                map,
                            ).await;
                        }
                        None =>{

                        }
                    }
                }
            }
        });

        // loop {
        //     select! {
        //         read_data = read_fuse.next() => match read_data{
        //             Some(data) => {
        //                 let msg = data;
        //                 self.read_ch.0.send(msg);
        //             }
        //             None =>{
        //                 debug!("invalid stream data")
        //             }
        //         },

        //         read_data = self.add_ch.1.next() => match read_data{
        //             Some(data) => {
        //                 let map:BTreeMap<String,String>=BTreeMap::new();
        //                 let id: RedisResult<String> = con.xadd_map(
        //                     0, // maxlen,
        //                     "*",
        //                     map,
        //                 ).await;
        //             }
        //             None =>{

        //             }

        //         },
        //     }
    }
}
// }

// // futures_util::future::Fuse<Pin<Box<dyn futures_util::Future<Output = std::result::Result<std::string::String, RedisError>> + std::marker::Send>>>
// // futures_util::future::Fuse<Pin<Box<dyn futures_util::Future<Output = std::result::Result<std::string::String, RedisError>> + std::marker::Send>>>
// the method `next` exists for struct `futures_util::future::Fuse<Pin<Box<dyn futures_util::Future<Output = std::result::Result<std::string::String, RedisError>> + std::marker::Send>>>`, but its trait bounds were not satisfied
// the following trait bounds were not satisfied:
// `futures_util::future::Fuse<Pin<Box<dyn futures_util::Future<Output = std::result::Result<std::string::String, RedisError>> + std::marker::Send>>>: futures_util::Stream`
// which is required by `futures_util::future::Fuse<Pin<Box<dyn futures_util::Future<Output = std::result::Result<std::string::String, RedisError>> + std::marker::Send>>>: futures_util::StreamExt`rustcE0599
