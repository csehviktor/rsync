use thiserror::Error;

pub type RsyncResult<T> = Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Persistance(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error("r2 authentication error: {message}")]
    R2 { status: u16, message: String },
}
