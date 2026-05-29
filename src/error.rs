//! Typed error hierarchy for the Antigravity SDK.

use serde::{Deserialize, Serialize};

/// Structured detail for a single validation failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationDetail {
    /// Location path segments indicating where the error occurred.
    pub loc: Vec<String>,
    /// Human-readable error message.
    pub msg: String,
    /// Machine-readable error type tag.
    pub error_type: String,
}

/// Unified error type for the Antigravity SDK.
#[derive(Debug, Clone, thiserror::Error)]
pub enum AntigravityError {
    /// A network or transport-level failure.
    #[error("Connection error: {0}")]
    Connection(String),

    /// A server-side or agent execution failure.
    #[error("Execution error: {0}")]
    Execution(String),

    /// One or more input validation failures.
    #[error("Validation error: {message}")]
    Validation {
        /// Summary message for the validation error.
        message: String,
        /// Individual validation failures.
        errors: Vec<ValidationDetail>,
    },
}
