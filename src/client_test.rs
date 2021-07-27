#[cfg(test)]
mod tests {
    use crate::server::RedisServer;
    use crate::{client::*, bus::*};
    use simple_logger::SimpleLogger;

    #[tokio::test]
    async fn test_read_and_add() {
        SimpleLogger::new().init().unwrap();

        // let server = RedisServer::new("127.0.0.1".to_owned(), "6379".to_owned());

        let mut client = RedisClient::new("localhost:6379")
            .unwrap()
            .with_consumer_name("consumer_1")
            .with_group_name("group_1");

        let mut read_rx = client.read(&vec!["key_1".to_string(), "key_2".to_string()]).unwrap();

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
        client.add(&msg1).unwrap();
        client.add(&msg2).unwrap();

        if let Some(stream) = read_rx.recv().await {
            println!("Received {:?}", stream)
        }

        if let Some(stream) = read_rx.recv().await {
            println!("Received {:?}", stream)
        }
    }
}
