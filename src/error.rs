use redis::RedisError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid AddressID")]
    RedisError{msg: String},

    #[error("Invalid Stream ID")]
    InvalidStreamID,
}

impl From<RedisError> for Error {
    fn from(err: RedisError) -> Self {
        Error::RedisError{msg: format!("{}", err)}
    }
}

pub type Result<T> = std::result::Result<T, Error>;
