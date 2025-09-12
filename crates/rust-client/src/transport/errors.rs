use thiserror::Error;

// TODO
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("transport error: {0}")]
    Other(#[from] anyhow::Error),
}
