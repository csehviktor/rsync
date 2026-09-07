use thiserror::Error;

pub type RsyncResult<T> = Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("r2 error: {0}")]
    R2(String),

    #[error("{0}")]
    Persistance(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Xml(#[from] serde_xml_rs::Error),

    #[error(transparent)]
    Http(#[from] reqwest::Error),
}
