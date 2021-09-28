use crate::stream::Stream;
use async_trait::async_trait;
use futures::channel::mpsc::Sender;

#[async_trait]
pub trait StreamBus {
    fn xadd_sender(&self) -> Sender<Stream>;
    fn xack_sender(&self) -> Sender<Stream>;
    async fn run<'a>(mut self, keys: &[&'a str], mut read_tx: Sender<Stream>);
}
