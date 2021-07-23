use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid AddressID")]
    InvalidAddressID,
}

pub type Result<T> = std::result::Result<T, Error>;
