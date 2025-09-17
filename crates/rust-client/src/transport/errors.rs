use thiserror::Error;

// TODO
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("transport layer is not enabled")]
    Disabled,
    #[error("transport error: {0}")]
    Other(#[from] anyhow::Error),
}
