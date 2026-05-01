//! Register slot types shared across the application.

use serde::{Deserialize, Serialize};

/// Named measurement slots for comparison and reference tracking.
///
/// Phase 1: Current, Reference, W.
/// Phase 2 extends with K, R, G, B, C, M, Y.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegisterSlot {
    /// The most recent measurement (auto-populated on every read).
    Current,
    /// The reference against which DeltaE is computed.
    Reference,
    /// White point measurement; density baseline for print workflows.
    W,
}
