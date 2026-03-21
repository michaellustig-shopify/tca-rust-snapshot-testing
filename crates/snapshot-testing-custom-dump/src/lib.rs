//! ┌─────────────────────────────────────────────────────┐
//! │  SNAPSHOT-TESTING-CUSTOM-DUMP                        │
//! │  Pretty-print integration for readable snapshots    │
//! ├─────────────────────────────────────────────────────┤
//! │                                                      │
//! │  struct User { name: "Alice", age: 30 }              │
//! │       │                                              │
//! │       ▼                                              │
//! │  custom_dump(&user)                                  │
//! │       │                                              │
//! │       ▼                                              │
//! │  "User {\n  name: \"Alice\",\n  age: 30\n}"         │
//! │                                                      │
//! ├─────────────────────────────────────────────────────┤
//! │  WHY: Debug output is ugly and unstable across       │
//! │  Rust versions. Custom dump provides deterministic,  │
//! │  human-readable serialization for snapshot testing.  │
//! │                                                      │
//! │  SWIFT EQUIVALENT: SnapshotTestingCustomDump         │
//! │  (integrates with swift-custom-dump library)         │
//! │                                                      │
//! │  TESTED BY: tests/custom_dump_tests.rs               │
//! │  EDGE CASES: nested types, collections, enums,       │
//! │  optional fields, recursive structures               │
//! │                                                      │
//! │  CHANGELOG:                                          │
//! │  • v0.1.0 — Initial stub                             │
//! │                                                      │
//! │  HISTORY: git log --oneline --follow -- crates/snapshot-testing-custom-dump/
//! └─────────────────────────────────────────────────────┘

pub use snapshot_testing;

// TODO: Implement custom dump serialization and Snapshotting strategy
