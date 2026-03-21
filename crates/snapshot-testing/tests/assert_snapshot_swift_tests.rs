//! Tests ported from AssertSnapshotSwiftTests.swift
//!
//! This file tests the Swift Testing integration path (as opposed to XCTest).
//! In Swift, these tests live inside `BaseSuite` extensions and use the
//! `@Test` attribute from Swift Testing. In Rust we just have regular `#[test]`.
//!
//! Coverage:
//! - Basic dump snapshotting with record: .missing
//! - Main-actor annotated tests (no Rust equivalent, but we port the intent)

#[cfg(test)]
mod assert_snapshot_swift_tests {
    #[allow(unused_imports)]
    use snapshot_testing::{
        assert_snapshot, Snapshotting,
    };

    /// Verifies that a struct can be snapshotted using the dump strategy with
    /// `record: .missing` configuration. This was the Swift Testing variant
    /// of the basic dump test.
    /// Swift: `BaseSuite.AssertSnapshotTests.dump()`
    #[test]
    #[ignore] // TODO: implement Snapshotting::dump()
    fn test_dump() {
        #[derive(Debug)]
        #[allow(dead_code)]
        struct User {
            id: i32,
            name: String,
            bio: String,
        }
        let _user = User {
            id: 1,
            name: "Blobby".into(),
            bio: "Blobbed around the world.".into(),
        };
        // with_snapshot_testing(record: .missing) {
        //     assertSnapshot(of: user, as: .dump)
        // }
    }

    /// Verifies that snapshot assertions work correctly when called from a
    /// MainActor context. In Rust there is no actor isolation, but this test
    /// ensures the same dump output is produced regardless of execution context.
    /// Swift: `BaseSuite.MainActorTests.dump()`
    #[test]
    #[ignore] // TODO: implement Snapshotting::dump()
    fn test_main_actor_dump() {
        #[derive(Debug)]
        #[allow(dead_code)]
        struct User {
            id: i32,
            name: String,
            bio: String,
        }
        let _user = User {
            id: 1,
            name: "Blobby".into(),
            bio: "Blobbed around the world.".into(),
        };
        // assertSnapshot(of: user, as: .dump)
    }
}
