//! ┌─────────────────────────────────────────────────────┐
//! │  lib.rs — Trinity library root                      │
//! ├─────────────────────────────────────────────────────┤
//! │  WHAT: Re-exports all Trinity modules so they are   │
//! │  available for doc tests and integration tests.     │
//! │  The actual binary entry point is main.rs.          │
//! │                                                     │
//! │  WHY: Rust doc tests only run on library crates.    │
//! │  By having a lib.rs that re-exports everything,     │
//! │  `cargo test --doc -p trinity` works correctly.     │
//! │                                                     │
//! │  ALTERNATIVES:                                      │
//! │  • No lib.rs — doc tests won't run                  │
//! │  • Separate library crate — more indirection        │
//! │                                                     │
//! │  TESTED BY: All doc tests across all modules        │
//! │                                                     │
//! │  EDGE CASES: None — this is just re-exports.        │
//! │                                                     │
//! │  CHANGELOG:                                         │
//! │  • v0.1.0 — Initial module structure                │
//! │                                                     │
//! │  HISTORY: git log --oneline --follow -- lib.rs      │
//! └─────────────────────────────────────────────────────┘

/// Error types for Trinity operations.
pub mod error;

/// Persistent state management (.trinity/state.json).
pub mod state;

/// Rust source code scanning and parsing with `syn`.
pub mod scanner;

/// The `trinity init` command — workspace setup and hook installation.
pub mod init;

/// The `trinity check` command — three-way sync verification.
pub mod check;

/// The `trinity status` command — human-readable state display.
pub mod status;
