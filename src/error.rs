#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("HTTP request failed")]
    HttpError(#[from] attohttpc::Error),
    #[error("Unable to get any server")]
    NoServer,
    #[error("Invalid packet header")]
    InvalidHeader,
    #[error("I/O error")]
    IoError(#[from] std::io::Error)
}

pub type Result<T> = std::result::Result<T, Error>;