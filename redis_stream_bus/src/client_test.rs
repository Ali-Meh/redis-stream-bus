#[cfg(test)]
mod tests {
    use crate::bus;
    use crate::bus::{StreamBus, Stream, Message};
    use crate::client::RedisClient;
    use futures::channel::mpsc::channel;
    use futures::select;
    use futures::{SinkExt, StreamExt};
    use serde::{Deserialize,Serialize};
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::task::{self};

    const REDIS_CON: &str = "redis://localhost:6379";

    #[derive(Serialize, Deserialize, Debug, Clone, Default)]
    pub struct Drafted {
        pub id: String,
    }

    impl Drafted{
        fn new(id:&str)->Self{
            Drafted { id: id.to_string() }
        }
    }



    #[test]
    fn test_basic_message() {
        let payload=Drafted{
            id:"payload_id".to_string()
        };
        let stream = bus::Stream{
            id: Some("id".to_string()),
            key: "key".to_string(),
            message: bus::Message{
                fields: serde_json::to_value(payload.clone()).unwrap(),
            },
        };

        assert_eq!(stream.id, Some("id".to_string()));
        assert_eq!(stream.key, "key");
        let drafted:Drafted=serde_json::from_value(stream.message.fields).unwrap();
        assert_eq!(drafted.id, payload.id);
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Event {
        #[serde(flatten)]
        pub header: EventHeader,
        pub payload: EventPayload,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct EventHeader {
        pub wss_id: String,
        pub ws_id: String,
    }
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct EventPayload {
        pub payload:String
    }


    #[test]
    fn test_headered_message() {
        let payload=Event{
            header: EventHeader{ wss_id: "wss_id".to_string(), ws_id: "ws_id".to_string() },
            payload: EventPayload{ payload: "Hello There".to_string() },
        };
        let stream = Stream{
            id: Some("id".to_string()),
            key: "key".to_string(),
            message: Message{
                fields: serde_json::to_value(payload.clone()).unwrap(),
            },
        };

        assert_eq!(stream.id, Some("id".to_string()));
        assert_eq!(stream.key, "key");
        let drafted:Event=serde_json::from_value(stream.message.fields.clone()).unwrap();
        eprintln!("{}",serde_json::json!(stream.message.fields));
        assert_eq!(drafted.header.ws_id, payload.header.ws_id);
        assert_eq!(drafted.header.wss_id, payload.header.wss_id);
        assert_eq!(drafted.payload.payload, payload.payload.payload);
    }

    #[tokio::test]
    async fn test_add_stream_with_id() {
        let client = RedisClient::new(REDIS_CON, "group_1", "consumer_1").unwrap();

        let mut add_tx = client.xadd_sender();
        let (read_tx, mut read_rx) = channel(100);

        task::spawn(async move {
            client.run(&["key_read_id"], read_tx).await;
        });

        let mut stream_out = bus::Stream{
            id: None,
            key: "key_read_id".to_string(),
            message: Message{
                fields:serde_json::to_value(Drafted::default()).unwrap(),
            },
        };

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        stream_out.id = Some(format!("{}-0", since_the_epoch.as_millis()));

        add_tx.send(stream_out.clone()).await.unwrap();

        let stream_in = read_rx.next().await.unwrap();

        assert_eq!(stream_in.id, stream_out.id);
    }

    #[tokio::test]
    async fn test_read_one_group() {
        let client = RedisClient::new(REDIS_CON, "group_1", "consumer_1").unwrap();

        let mut add_tx = client.xadd_sender();
        let (read_tx, mut read_rx) = channel(100);

        task::spawn(async move {
            client.run(&["key_read_one_group"], read_tx).await;
        });

        let stream_1 = bus::Stream::new("key_read_one_group",serde_json::to_value(Drafted::default()).unwrap());

        add_tx.send(stream_1.clone()).await.unwrap();

        let stream_in = read_rx.next().await.unwrap();

        assert_eq!(stream_in.message, stream_1.message);
        assert_eq!(stream_in.key, stream_1.key);
    }

    #[tokio::test]
    async fn test_read_two_groups() {
        let client_1 = RedisClient::new(REDIS_CON, "group_1", "consumer_1").unwrap();
        let client_2 = RedisClient::new(REDIS_CON, "group_2", "consumer_1").unwrap();

        let mut add_tx_1 = client_1.xadd_sender();
        let (read_tx_1, mut read_rx_1) = channel(100);
        let (read_tx_2, mut read_rx_2) = channel(100);

        task::spawn(async move {
            client_1.run(&["foo"], read_tx_1).await;
        });

        task::spawn(async move {
            client_2.run(&["foo", "bar"], read_tx_2).await;
        });

        let stream_1 = bus::Stream::new("zoo", serde_json::to_value(Drafted::default()).unwrap());
        let stream_2 = bus::Stream::new("foo", serde_json::to_value(Drafted::default()).unwrap());
        let stream_3 = bus::Stream::new("bar", serde_json::to_value(Drafted::default()).unwrap());

        add_tx_1.send(stream_1.clone()).await.unwrap();
        add_tx_1.send(stream_2.clone()).await.unwrap();
        add_tx_1.send(stream_3.clone()).await.unwrap();

        let streams_in_1 = read_rx_1.next().await.unwrap();
        let streams_in_2 = read_rx_2.next().await.unwrap();
        let streams_in_3 = read_rx_2.next().await.unwrap();

        assert_eq!(streams_in_1.key, "foo");
        assert_eq!(streams_in_2.key, "foo");
        assert_eq!(streams_in_3.key, "bar");
    }

    #[tokio::test]
    async fn test_two_consumers() {
        let consumer_1 = RedisClient::new(REDIS_CON, "group_1", "consumer_1").unwrap();
        let consumer_2 = RedisClient::new(REDIS_CON, "group_1", "consumer_2").unwrap();

        let mut add_tx_1 = consumer_1.xadd_sender();
        let (read_tx_1, mut read_rx_1) = channel(100);
        let (read_tx_2, mut read_rx_2) = channel(100);

        task::spawn(async move {
            consumer_1.run(&["key_2_consumers"], read_tx_1).await;
        });

        task::spawn(async move {
            consumer_2.run(&["key_2_consumers"], read_tx_2).await;
        });

        let streams = [
            bus::Stream::new("key_2_consumers", serde_json::to_value(Drafted::new("1")).unwrap()),
            bus::Stream::new("key_2_consumers", serde_json::to_value(Drafted::new("2")).unwrap()),
            bus::Stream::new("key_2_consumers", serde_json::to_value(Drafted::new("3")).unwrap()),
            bus::Stream::new("key_2_consumers", serde_json::to_value(Drafted::new("4")).unwrap()),
        ];

        add_tx_1.send(streams[0].clone()).await.unwrap();
        add_tx_1.send(streams[1].clone()).await.unwrap();
        add_tx_1.send(streams[2].clone()).await.unwrap();
        add_tx_1.send(streams[3].clone()).await.unwrap();

        let mut id = Vec::new();

        for _ in 0..4 {
            select! {
                read_option = read_rx_1.next() => if let Some(stream) = read_option {
                    id.push(serde_json::from_value::<Drafted>(stream.message.fields).unwrap().id)
                },
                read_option = read_rx_2.next() => if let Some(stream) = read_option {
                    id.push(serde_json::from_value::<Drafted>(stream.message.fields).unwrap().id)
                },
            }
        }

        assert!(id.contains(&"1".to_owned()));
        assert!(id.contains(&"2".to_owned()));
        assert!(id.contains(&"3".to_owned()));
        assert!(id.contains(&"4".to_owned()));
    }

    #[tokio::test]
    async fn test_ack() {
        let client_1 = RedisClient::new(REDIS_CON, "group_1", "consumer_1").unwrap();

        let mut add_tx = client_1.xadd_sender();
        let mut ack_tx = client_1.xack_sender();
        let (read_tx_1, mut read_rx_1) = channel(100);

        task::spawn(async move {
            client_1.run(&["key_ack"], read_tx_1).await;
        });

        let stream_1 = bus::Stream::new("key_ack", serde_json::to_value(Drafted::new("1")).unwrap());

        let stream_2 = bus::Stream::new("key_ack", serde_json::to_value(Drafted::new("2")).unwrap());

        add_tx.send(stream_1.clone()).await.unwrap();
        let stream_1 = read_rx_1.next().await.unwrap();
        ack_tx.send(stream_1.clone()).await.unwrap();

        add_tx.send(stream_2.clone()).await.unwrap();
        read_rx_1.next().await.unwrap();
    }
}
