#[cfg(test)]
mod tests {
    use crate::bus;
    use crate::bus::StreamBus;
    use crate::client::RedisClient;
    use futures::channel::mpsc::channel;
    use futures::select;
    use futures::{SinkExt, StreamExt};
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::task::{self};

    const REDIS_CON: &str = "redis://localhost:6379";

    #[test]
    fn test_new_stream() {
        let stream = bus::Stream::new("key", "module", None, "message");

        assert_eq!(stream.key, "key");
        assert_eq!(stream.value.request_id, None);
        assert_eq!(stream.value.module, "module");
        assert_eq!(stream.value.message, "\"message\"");
    }

    #[tokio::test]
    async fn test_add_stream_with_id() {
        let client = RedisClient::new(REDIS_CON, "group_1", "consumer_1").unwrap();

        let mut add_tx = client.xadd_sender();
        let (read_tx, mut read_rx) = channel(100);

        task::spawn(async move {
            client.run(&["key_read_id"], read_tx).await;
        });

        let mut stream_out = bus::Stream::new("key_read_id", "module_1", None, "message_1");

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

        let stream_1 = bus::Stream::new("key_read_one_group", "module_1", None, "message");

        add_tx.send(stream_1.clone()).await.unwrap();

        let stream_in = read_rx.next().await.unwrap();

        assert_eq!(stream_in.value.message, stream_1.value.message);
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

        let stream_1 = bus::Stream::new("zoo", "module_1", None, "message_1");
        let stream_2 = bus::Stream::new("foo", "module_2", None, "message_2");
        let stream_3 = bus::Stream::new("bar", "module_3", None, "message_3");

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
            bus::Stream::new("key_2_consumers", "module_1", None, "message_1"),
            bus::Stream::new("key_2_consumers", "module_2", None, "message_2"),
            bus::Stream::new("key_2_consumers", "module_3", None, "message_3"),
            bus::Stream::new("key_2_consumers", "module_4", None, "message_4"),
        ];

        add_tx_1.send(streams[0].clone()).await.unwrap();
        add_tx_1.send(streams[1].clone()).await.unwrap();
        add_tx_1.send(streams[2].clone()).await.unwrap();
        add_tx_1.send(streams[3].clone()).await.unwrap();

        let mut modules = Vec::new();

        for _ in 0..4 {
            select! {
                read_option = read_rx_1.next() => if let Some(stream) = read_option {
                    modules.push(stream.value.module)
                },
                read_option = read_rx_2.next() => if let Some(stream) = read_option {
                    modules.push(stream.value.module)
                },
            }
        }

        assert!(modules.contains(&"module_1".to_owned()));
        assert!(modules.contains(&"module_2".to_owned()));
        assert!(modules.contains(&"module_3".to_owned()));
        assert!(modules.contains(&"module_4".to_owned()));
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

        let stream_1 = bus::Stream::new("key_ack", "module_1", None, "message_1");

        let stream_2 = bus::Stream::new("key_ack", "module_1", None, "message_2");

        add_tx.send(stream_1.clone()).await.unwrap();
        let stream_1 = read_rx_1.next().await.unwrap();
        ack_tx.send(stream_1.clone()).await.unwrap();

        add_tx.send(stream_2.clone()).await.unwrap();
        read_rx_1.next().await.unwrap();
    }
}
