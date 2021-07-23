#[cfg(test)]
mod tests {
    use crate::server::RedisServer;
    use crate::{client::*, provider::*};

    #[tokio::test]
    async fn test_read_and_add() {
        let server = RedisServer::new("127.0.0.1".to_owned(), "6379".to_owned());

        let client = RedisClient::new(server.get_client_addr())
            .with_consumer_name("consumer_1")
            .with_group_name("group_1");

        let add_tx = client.add_transmitter();
        let read_rx = client.read_receiver();

        client.run();

        let msg1 = Stream {
            id: None,
            key: "key_1".to_owned(),
            value: StreamValue {
                module: "module_1".to_owned(),
                request_id: None,
                message: "message_1".to_owned(),
            },
        };

        let msg2 = Stream {
            id: None,
            key: "key_2".to_owned(),
            value: StreamValue {
                module: "module_2".to_owned(),
                request_id: None,
                message: "message_2".to_owned(),
            },
        };
        add_tx.send(msg1).await.unwrap();
        add_tx.send(msg2).await.unwrap();

        loop {
            let stream = read_rx.recv().await.unwrap();
            print!("{:?}", stream)
        }
    }
}
