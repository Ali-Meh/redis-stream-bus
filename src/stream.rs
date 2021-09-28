use std::collections::HashMap;

///
/// Stream is the data structure to keep streams nice and tidy
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stream {
    /// redis streams Id auto added via redis node
    pub id: Option<String>,
    /// redis stream this streams belongs to
    pub key: String,
    /// data embedded in the event
    pub message: HashMap<String, redis::Value>,
}

impl Stream {
    pub fn new(
        key: &str,
        id: Option<String>,
        message: HashMap<String, redis::Value>,
    ) -> Self {
        Stream {
            id: id,
            key: key.to_owned(),
            message: message,
        }
    }
}
