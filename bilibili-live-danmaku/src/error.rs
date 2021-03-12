#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid packet header")]
    InvalidHeader,
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
    #[error("WebSocket error")]
    WsError(#[from] tokio_tungstenite::tungstenite::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
