#[derive(Debug, thiserror::Error)]
pub enum OssError {
    #[error("http error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("json error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("base64 decode error: {0}")]
    DecodeError(#[from] base64::DecodeError),
    #[error("{0}")]
    Err(String),
}
