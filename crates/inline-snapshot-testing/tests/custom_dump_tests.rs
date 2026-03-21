//! Tests ported from Swift's CustomDumpTests.swift
//!
//! Original: Tests/InlineSnapshotTestingTests/CustomDumpTests.swift
//!
//! These tests exercise the `customDump` snapshotting strategy, which produces
//! a structured, human-readable dump of a value (similar to Swift's
//! `swift-custom-dump` library). This strategy is provided by the
//! `snapshot-testing-custom-dump` crate.
//!
//! In Swift, the test lives under `BaseSuite.CustomDumpSnapshotTests` and
//! imports both `InlineSnapshotTesting` and `SnapshotTestingCustomDump`.

// TODO: replace with real macro/function once inline snapshot rewriting is implemented
macro_rules! assert_inline_snapshot {
    ($value:expr, $strategy:expr, $expected:expr) => {
        let _ = ($value, $strategy, $expected);
    };
}

/// Placeholder for the `.customDump` snapshotting strategy.
/// In Swift this is `Snapshotting<Value, String>.customDump` which uses
/// `swift-custom-dump` to produce a structured text representation of any value.
/// The Rust equivalent will use a custom Debug-like formatter that produces
/// deterministic, readable output (not Rust's default Debug which varies by version).
fn strategy_custom_dump() -> &'static str {
    "custom_dump"
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that the `.customDump` strategy produces a structured,
    /// human-readable text representation of a struct. The output shows
    /// field names, values, and proper indentation.
    ///
    /// In the Swift version, the test creates a local `User` struct with
    /// `id`, `name`, and `bio` fields, then asserts the custom dump output
    /// matches a specific format:
    ///
    /// ```text
    /// User(
    ///   id: 1,
    ///   name: "Blobby",
    ///   bio: "Blobbed around the world."
    /// )
    /// ```
    ///
    /// The Rust equivalent will need a custom dump implementation that
    /// produces similarly readable output (not Rust's default `Debug` trait
    /// which uses different formatting conventions).
    ///
    /// Swift: `@Test func basics()`
    #[test]
    #[ignore] // TODO: implement inline snapshot
    fn test_basics() {
        #[allow(dead_code)]
        struct User {
            id: i32,
            name: String,
            bio: String,
        }

        let user = User {
            id: 1,
            name: "Blobby".to_string(),
            bio: "Blobbed around the world.".to_string(),
        };

        assert_inline_snapshot!(
            user,
            strategy_custom_dump(),
            r#"User(
  id: 1,
  name: "Blobby",
  bio: "Blobbed around the world."
)"#
        );
    }
}
