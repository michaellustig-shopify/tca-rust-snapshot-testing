//! ┌─────────────────────────────────────────────────────┐
//! │  SNAPSHOT-TESTING                                    │
//! │  Core diffing & assertion engine for snapshot tests  │
//! ├─────────────────────────────────────────────────────┤
//! │                                                      │
//! │   Value ──► Snapshotting<V,F> ──► Format             │
//! │                    │                  │               │
//! │                    ▼                  ▼               │
//! │              snapshot(value)    Diffing<F>            │
//! │                    │           ┌──┴──┐               │
//! │                    ▼           ▼     ▼               │
//! │              Format data    toData fromData          │
//! │                    │           │                      │
//! │                    ▼           ▼                      │
//! │              Write/Compare ◄──┘                       │
//! │              to __snapshots__/                        │
//! │                                                      │
//! ├─────────────────────────────────────────────────────┤
//! │  WHY: This is the foundation of the snapshot         │
//! │  testing system. All strategies, diffing, and        │
//! │  assertions flow through this crate.                 │
//! │                                                      │
//! │  ALTERNATIVES: `insta` crate (great but different    │
//! │  philosophy — we port the Point-Free composable      │
//! │  strategy pattern). `expectest` (too minimal).       │
//! │                                                      │
//! │  TESTED BY: crates/snapshot-testing/tests/           │
//! │  EDGE CASES: empty snapshots, unicode, binary data,  │
//! │  missing snapshot files, concurrent test execution   │
//! │                                                      │
//! │  CHANGELOG:                                          │
//! │  • v0.1.0 — Initial workspace setup, stub types      │
//! │                                                      │
//! │  HISTORY: git log --oneline --follow -- crates/snapshot-testing/
//! └─────────────────────────────────────────────────────┘

pub mod assert;
pub mod config;
pub mod diff;
pub mod diffing;
pub mod snapshotting;
pub mod strategies;

// Re-export key types at crate root
pub use assert::{assert_snapshot, verify_snapshot};
pub use config::{DiffTool, Record, SnapshotTestingConfiguration};
pub use diff::line_diff;
pub use diffing::Diffing;
pub use snapshotting::Snapshotting;

// Re-export strategy modules so users can reference them for documentation,
// though strategies are accessed as associated functions on Snapshotting
// (e.g., Snapshotting::lines(), Snapshotting::json()).
pub use strategies::{data, debug, description, json, lines};
