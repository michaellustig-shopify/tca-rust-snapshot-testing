//! Tests ported from SwiftTestingTests.swift
//!
//! These tests verify the Swift Testing framework integration.
//! In Swift, they check that snapshot mismatches produce proper `Issue`s
//! via `withKnownIssue`. In Rust, the equivalent is verifying that
//! `verify_snapshot` returns an appropriate error.
//!
//! Coverage:
//! - Basic snapshot match and intentional mismatch with diff verification
//! - Image-based snapshot mismatch detection (UIImage/NSImage)

#[cfg(test)]
mod swift_testing_tests {
    #[allow(unused_imports)]
    use snapshot_testing::{
        verify_snapshot, Snapshotting,
    };

    /// Verifies that a matching snapshot succeeds and a mismatched snapshot
    /// produces a diff containing the expected change markers.
    ///
    /// The Swift test:
    /// 1. Snapshots `["Hello", "World"]` with name "snap" (should match reference)
    /// 2. Snapshots `["Goodbye", "World"]` with same name (should mismatch)
    /// 3. Asserts the mismatch diff contains `-"Hello"` / `+"Goodbye"`
    ///
    /// Swift: `SwiftTestingTests.testSnapshot()`
    #[test]
    #[ignore] // TODO: implement Snapshotting::dump() and diff verification
    fn test_snapshot() {
        // let hello = vec!["Hello", "World"];
        // let goodbye = vec!["Goodbye", "World"];
        //
        // // First assertion should match the reference
        // assert_snapshot(&hello, &Snapshotting::dump(), Some("snap"), ...);
        //
        // // Second assertion should fail with a diff showing Hello → Goodbye
        // let result = verify_snapshot(&goodbye, &Snapshotting::dump(), Some("snap"), ...);
        // assert!(result.is_err());
        // let err_msg = result.unwrap_err().to_string();
        // assert!(err_msg.contains("-\"Hello\""));
        // assert!(err_msg.contains("+\"Goodbye\""));
    }

    /// Verifies that comparing two different UIImages (red vs blue pixel) produces
    /// a mismatch error saying "does not match reference".
    /// Apple-only: UIImage is not available in Rust.
    /// Swift: `SwiftTestingTests.testUIImage()`
    #[test]
    #[ignore] // TODO: implement image snapshotting (Apple only, UIKit)
    fn test_ui_image() {
        // Red 1x1 pixel image vs blue 1x1 pixel image
        // First snapshot (red) should match reference
        // Second snapshot (blue) should fail: "does not match reference"
    }

    /// Verifies that comparing two different NSImages (red vs blue pixel) produces
    /// a mismatch error saying "does not match reference".
    /// Apple-only: NSImage is not available in Rust.
    /// Swift: `SwiftTestingTests.testNSImage()`
    #[test]
    #[ignore] // TODO: implement image snapshotting (Apple only, AppKit)
    fn test_ns_image() {
        // Red 1x1 pixel image vs blue 1x1 pixel image (macOS NSImage variant)
        // Same logic as test_ui_image but using AppKit types
    }
}
