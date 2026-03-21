//! ┌─────────────────────────────────────────────────────┐
//! │  INLINE-SNAPSHOT-TESTING                             │
//! │  Snapshots embedded directly in test source code     │
//! ├─────────────────────────────────────────────────────┤
//! │                                                      │
//! │  assert_inline_snapshot!(value, @"expected output")   │
//! │       │                                              │
//! │       ├──► snapshot = strategy.snapshot(value)        │
//! │       │                                              │
//! │       ├──► compare with inline string literal        │
//! │       │    └── if match: pass                        │
//! │       │    └── if mismatch: update source file       │
//! │       │                                              │
//! │       └──► return pass/fail                          │
//! │                                                      │
//! ├─────────────────────────────────────────────────────┤
//! │  WHY: Inline snapshots avoid managing separate       │
//! │  __snapshots__/ files. The expected output lives     │
//! │  right next to the assertion, making it easy to      │
//! │  review in code review.                              │
//! │                                                      │
//! │  SWIFT EQUIVALENT: InlineSnapshotTesting module      │
//! │  (Uses swift-syntax to modify source files)          │
//! │                                                      │
//! │  RUST APPROACH: Uses `insta`-style inline snapshots  │
//! │  or proc-macro source rewriting.                     │
//! │                                                      │
//! │  TESTED BY: tests/inline_tests.rs                    │
//! │  EDGE CASES: multiline strings, special characters,  │
//! │  nested macro invocations, source file encoding      │
//! │                                                      │
//! │  CHANGELOG:                                          │
//! │  • v0.1.0 — Initial stub                             │
//! │                                                      │
//! │  HISTORY: git log --oneline --follow -- crates/inline-snapshot-testing/
//! └─────────────────────────────────────────────────────┘

pub use snapshot_testing;

// TODO: Implement inline snapshot assertion macro and source rewriting
