use serde::Serialize;
use bus_derive::*;
use redis_stream_bus::StreamParsable;


#[derive(Serialize, Debug, Clone, RedisStream)]
#[stream_name("p2p:adv:drafted")]
pub struct Drafted {
    pub id: String,
}



fn main(){
    let drafted=Drafted{
        id:"".to_string()
    };

    println!("{}",drafted.key());
    // println!("{}",drafted.Serialize());

    let _=drafted;
}