use thiserror::Error;

#[derive(Debug, Error)]
pub enum DisplayError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Protocol error: {0}")]
    ProtocolError(String),
}

#[derive(Debug, Error)]
pub enum MeterError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Read timeout")]
    ReadTimeout,
}

#[derive(Debug, Error)]
pub enum PatternGenError {
    #[error("Display error: {0}")]
    DisplayError(String),
}
