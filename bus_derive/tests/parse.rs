use serde::Serialize;
use bus_derive::*;
use redis_stream_bus::StreamParsable;

#[derive(Serialize, Debug, Clone, RedisStream)]
#[stream_name("test")]
pub struct TestStruct {
    pub id: String,
}

fn main() {
    let test_struct=TestStruct{
        id:"1".to_string()
    };


    assert_eq!(test_struct.key(),TestStruct::name.to_string());
}
