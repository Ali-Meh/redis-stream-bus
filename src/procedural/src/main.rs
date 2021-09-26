use serde::Serialize;
use procedural::*;


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

    let _=drafted;
}