#[cfg(test)]
mod tests {
    use serde::{Serialize,Deserialize};
    use redis_stream_bus::*;
    
    
    #[derive(Serialize,Deserialize, Debug, Clone, RedisStream)]
    #[stream_name("p2p:adv:drafted")]
    pub struct Drafted {
        pub id: String,
    }
    

    #[test]
    fn serialize_drafted_with_key_exposed() {
        let drafted=Drafted{
            id:"".to_string()
        };
    
        assert_eq!(drafted.key(),Drafted::name);
    }
}
