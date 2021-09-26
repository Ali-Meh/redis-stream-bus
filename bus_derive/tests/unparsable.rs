use bus_derive::*;

#[derive(Debug, Clone, RedisStream)]
#[stream_name("test")]
pub struct TestStruct {
    pub id: String,
}

fn main() {}
