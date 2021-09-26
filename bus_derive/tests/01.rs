use serde::Serialize;
use redis_stream_bus::StreamParsable;
use bus_derive::*;


#[derive(Serialize, Debug, Clone, RedisStream)]
#[stream_name("p2p:adv:drafted")]
pub struct Drafted {
    pub id: String,
}

fn main() {}
