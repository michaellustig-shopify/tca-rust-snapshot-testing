//! ┌─────────────────────────────────────────────────────┐
//! │  error.rs — Trinity error types                     │
//! ├─────────────────────────────────────────────────────┤
//! │  WHAT: Defines all error variants Trinity can       │
//! │  produce using thiserror for ergonomic Display/     │
//! │  Error impl generation.                             │
//! │                                                     │
//! │  WHY: Centralizing errors in one enum lets every    │
//! │  module return TrinityError via `?` without manual  │
//! │  mapping boilerplate.                               │
//! │                                                     │
//! │  ALTERNATIVES:                                      │
//! │  • anyhow — more convenient but loses typed errors  │
//! │  • manual impls — verbose, error-prone              │
//! │                                                     │
//! │  TESTED BY: compile-time only (derives guarantee    │
//! │  correctness)                                       │
//! │                                                     │
//! │  EDGE CASES: IO errors from missing files, JSON     │
//! │  parse failures on corrupted state.json, syn parse  │
//! │  failures on invalid Rust source.                   │
//! │                                                     │
//! │  CHANGELOG:                                         │
//! │  • v0.1.0 — Initial error variants                  │
//! │                                                     │
//! │  HISTORY: git log --oneline --follow -- error.rs    │
//! └─────────────────────────────────────────────────────┘

use std::path::PathBuf;

/// All errors that Trinity can produce.
///
/// Each variant wraps an underlying cause (IO, JSON, syn parse, etc.)
/// so callers get a clear message about what went wrong and where.
///
/// # Examples
///
/// ```
/// use trinity::error::TrinityError;
///
/// let err = TrinityError::NotInitialized;
/// assert!(err.to_string().contains("not initialized"));
/// ```
#[derive(Debug, thiserror::Error)]
pub enum TrinityError {
    /// Trinity has not been initialized in this workspace.
    #[error("Trinity is not initialized. Run `trinity init` first.")]
    NotInitialized,

    /// An IO operation failed (file read, directory walk, etc.).
    #[error("IO error at {path:?}: {source}")]
    Io {
        /// The path involved in the failed IO operation.
        path: PathBuf,
        /// The underlying IO error.
        source: std::io::Error,
    },

    /// JSON serialization or deserialization failed.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Failed to parse a Rust source file with syn.
    #[error("Rust parse error in {path:?}: {message}")]
    Parse {
        /// The file that failed to parse.
        path: PathBuf,
        /// The parse error description.
        message: String,
    },

    /// Running a git command failed.
    #[error("Git command failed: {0}")]
    Git(String),

    /// The user cancelled an operation.
    #[error("Operation cancelled by user.")]
    Cancelled,
}

/// Convenience alias for `Result<T, TrinityError>`.
pub type TrinityResult<T> = Result<T, TrinityError>;

/// Helper to convert a `std::io::Error` into `TrinityError::Io` with a path.
///
/// # Examples
///
/// ```
/// use trinity::error::io_err;
/// use std::path::PathBuf;
///
/// let io = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
/// let err = io_err(PathBuf::from("/tmp/missing.rs"), io);
/// assert!(err.to_string().contains("missing.rs"));
/// ```
pub fn io_err(path: PathBuf, source: std::io::Error) -> TrinityError {
    TrinityError::Io { path, source }
}
