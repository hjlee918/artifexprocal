//! Pattern generator trait — for PGenerator, LG internal TPG, etc.

use crate::types::Patch;
use std::fmt;

/// Error type for pattern generator operations.
#[derive(Debug, Clone, PartialEq)]
pub enum PatternGenError {
    ConnectionFailed,
    PatchFailed,
    Timeout,
    Other(String),
}

impl fmt::Display for PatternGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternGenError::ConnectionFailed => {
                write!(f, "Failed to connect to pattern generator")
            }
            PatternGenError::PatchFailed => write!(f, "Failed to display patch"),
            PatternGenError::Timeout => write!(f, "Pattern generator timed out"),
            PatternGenError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for PatternGenError {}

/// Abstract interface for pattern generators.
pub trait PatternGenerator: Send {
    /// Self-test: verify connectivity.
    fn probe(&mut self) -> Result<bool, PatternGenError>;

    /// Display a test patch.
    fn display_patch(&mut self,
        patch: Patch,
    ) -> Result<(), PatternGenError>;

    /// Display black (clear the patch).
    fn display_black(&mut self) -> Result<(), PatternGenError> {
        self.display_patch(Patch::rgb8(0, 0, 0))
    }

    /// Disconnect from the pattern generator.
    fn disconnect(&mut self) -> Result<(), PatternGenError> {
        Ok(())
    }
}
