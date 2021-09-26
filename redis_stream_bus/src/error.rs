use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {

    #[error("Connection error")]
    RedisError(#[from] redis::RedisError),

    #[error("Invalid Stream ID")]
    InvalidStreamID,
}


pub type Result<T> = std::result::Result<T, Error>;
