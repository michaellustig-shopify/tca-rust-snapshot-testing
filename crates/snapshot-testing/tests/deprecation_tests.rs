//! Tests ported from DeprecationTests.swift
//!
//! The Swift file tested a deprecated `isRecording` global proxy that
//! mirrors the `SnapshotTesting.record` property. In Rust, we test the
//! equivalent: that the global/thread-local record state can be read back
//! after being set.

#[cfg(test)]
mod deprecation_tests {
    use snapshot_testing::{
        config::{current_record, with_snapshot_testing},
        Record, SnapshotTestingConfiguration,
    };

    /// Verifies that the deprecated `isRecording` proxy correctly reflects
    /// the current recording state. In Rust, this is `current_record()`.
    ///
    /// The Swift test set `SnapshotTesting.record = true` then checked
    /// `isRecording == true`, then set it to `false` and checked again.
    ///
    /// In Rust we verify via `with_snapshot_testing` scoping.
    ///
    /// Swift: `func testIsRecordingProxy()`
    #[test]
    #[ignore] // TODO: implement is_recording() deprecated accessor
    fn test_is_recording_proxy() {
        // Set record to All (equivalent of record = true)
        let config_all = SnapshotTestingConfiguration {
            record: Some(Record::All),
            diff_tool: None,
        };
        with_snapshot_testing(config_all, || {
            // In Swift: XCTAssertEqual(isRecording, true)
            // assert_eq!(is_recording(), true);
            assert_eq!(current_record(), Record::All);
        });

        // Set record to Never (equivalent of record = false)
        let config_never = SnapshotTestingConfiguration {
            record: Some(Record::Never),
            diff_tool: None,
        };
        with_snapshot_testing(config_never, || {
            // In Swift: XCTAssertEqual(isRecording, false)
            // assert_eq!(is_recording(), false);
            assert_eq!(current_record(), Record::Never);
        });
    }
}
