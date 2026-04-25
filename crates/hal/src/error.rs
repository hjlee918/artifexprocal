use thiserror::Error;

#[derive(Debug, Error)]
pub enum MeterError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Read timeout")]
    ReadTimeout,
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

#[derive(Debug, Error)]
pub enum DisplayError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    #[error("Upload failed: {0}")]
    UploadFailed(String),
}

#[derive(Debug, Error)]
pub enum PatternGenError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Display error: {0}")]
    DisplayError(String),
}
