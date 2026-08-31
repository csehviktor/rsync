use thiserror::Error;

pub type RsyncResult<T> = Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Persistance(String),
}
