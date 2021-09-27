
pub trait StreamParsable : serde::Serialize {
    fn key(&self)-> String;
}