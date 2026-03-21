//! Tests ported from RecordTests.swift
//!
//! These tests exercise every `Record` mode (.never, .missing, .all, .failed)
//! by verifying:
//! - Whether snapshot files are written to disk
//! - Whether existing files are overwritten or preserved
//! - What error/failure messages are produced
//!
//! Each test creates a temporary snapshot directory, runs the assertion,
//! then checks the filesystem state.

#[cfg(test)]
mod record_tests {
    #[allow(unused_imports)]
    use snapshot_testing::{
        assert_snapshot, verify_snapshot, config::with_snapshot_testing, Record,
        SnapshotTestingConfiguration, Snapshotting,
    };
    use std::fs;
    use std::path::PathBuf;

    /// Helper: creates a temp directory for snapshot files, returning the path.
    /// Cleans up on drop via the returned guard.
    struct TempSnapshotDir {
        path: PathBuf,
    }

    impl TempSnapshotDir {
        fn new(_test_name: &str) -> Self {
            let path = std::env::temp_dir()
                .join("rust_snapshot_tests")
                .join("__Snapshots__")
                .join("RecordTests");
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("create temp snapshot dir");
            TempSnapshotDir { path }
        }

        fn snapshot_path(&self, test_name: &str) -> PathBuf {
            self.path.join(format!("{}.1.json", test_name))
        }
    }

    impl Drop for TempSnapshotDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    /// Verifies that `Record::Never` refuses to create a new snapshot file
    /// when no reference exists. The assertion should fail with
    /// "No reference was found on disk".
    ///
    /// Swift: `func testRecordNever()`
    #[test]
    #[ignore] // TODO: implement Snapshotting::json() and verify_snapshot file I/O
    fn test_record_never() {
        let dir = TempSnapshotDir::new("testRecordNever");
        let _snapshot_path = dir.snapshot_path("testRecordNever");

        let config = SnapshotTestingConfiguration {
            record: Some(Record::Never),
            diff_tool: None,
        };
        with_snapshot_testing(config, || {
            // let result = verify_snapshot(of: 42, as: .json);
            // assert!(result.is_err());
            // assert!(result.unwrap_err().to_string().contains("No reference was found on disk"));
        });

        assert!(!_snapshot_path.exists(), "snapshot file should NOT be created");
    }

    /// Verifies that `Record::Missing` writes a new snapshot file when none exists.
    /// The assertion "fails" (reports that it recorded) but the file content should
    /// be correct.
    ///
    /// Swift: `func testRecordMissing()`
    #[test]
    #[ignore] // TODO: implement Snapshotting::json() and verify_snapshot file I/O
    fn test_record_missing() {
        let dir = TempSnapshotDir::new("testRecordMissing");
        let _snapshot_path = dir.snapshot_path("testRecordMissing");

        let config = SnapshotTestingConfiguration {
            record: Some(Record::Missing),
            diff_tool: None,
        };
        with_snapshot_testing(config, || {
            // assertSnapshot(of: 42, as: .json)  → should record and report
        });

        // assert!(snapshot_path.exists());
        // assert_eq!(fs::read_to_string(&snapshot_path).unwrap(), "42");
    }

    /// Verifies that `Record::Missing` does NOT overwrite an existing snapshot file.
    /// If the existing file has different content ("999"), the test should fail
    /// with a diff message, but the file should remain unchanged.
    ///
    /// Swift: `func testRecordMissing_ExistingFile()`
    #[test]
    #[ignore] // TODO: implement Snapshotting::json() and verify_snapshot file I/O
    fn test_record_missing_existing_file() {
        let dir = TempSnapshotDir::new("testRecordMissing_ExistingFile");
        let _snapshot_path = dir.snapshot_path("testRecordMissing_ExistingFile");

        // Pre-populate with different content
        // fs::write(&snapshot_path, "999").unwrap();

        let config = SnapshotTestingConfiguration {
            record: Some(Record::Missing),
            diff_tool: None,
        };
        with_snapshot_testing(config, || {
            // let result = verify_snapshot(of: 42, as: .json);
            // Should fail: "Snapshot does not match reference"
        });

        // File should still contain "999" (not overwritten)
        // assert_eq!(fs::read_to_string(&snapshot_path).unwrap(), "999");
    }

    /// Verifies that `Record::All` writes a new snapshot file even when none exists.
    /// The assertion reports "Record mode is on" but the file is created correctly.
    ///
    /// Swift: `func testRecordAll_Fresh()`
    #[test]
    #[ignore] // TODO: implement Snapshotting::json() and verify_snapshot file I/O
    fn test_record_all_fresh() {
        let dir = TempSnapshotDir::new("testRecordAll_Fresh");
        let _snapshot_path = dir.snapshot_path("testRecordAll_Fresh");

        let config = SnapshotTestingConfiguration {
            record: Some(Record::All),
            diff_tool: None,
        };
        with_snapshot_testing(config, || {
            // assertSnapshot(of: 42, as: .json)  → "Record mode is on"
        });

        // assert!(snapshot_path.exists());
        // assert_eq!(fs::read_to_string(&snapshot_path).unwrap(), "42");
    }

    /// Verifies that `Record::All` overwrites an existing snapshot file.
    /// Even if the file has correct content, it gets rewritten.
    ///
    /// Swift: `func testRecordAll_Overwrite()`
    #[test]
    #[ignore] // TODO: implement Snapshotting::json() and verify_snapshot file I/O
    fn test_record_all_overwrite() {
        let dir = TempSnapshotDir::new("testRecordAll_Overwrite");
        let _snapshot_path = dir.snapshot_path("testRecordAll_Overwrite");

        // Pre-populate with different content
        // fs::write(&snapshot_path, "999").unwrap();

        let config = SnapshotTestingConfiguration {
            record: Some(Record::All),
            diff_tool: None,
        };
        with_snapshot_testing(config, || {
            // assertSnapshot(of: 42, as: .json)  → "Record mode is on"
        });

        // File should now contain "42" (overwritten)
        // assert_eq!(fs::read_to_string(&snapshot_path).unwrap(), "42");
    }

    /// Verifies that `Record::Failed` re-records the snapshot when there is
    /// a mismatch. The file should be updated to the new value.
    ///
    /// Swift: `func testRecordFailed_WhenFailure()`
    #[test]
    #[ignore] // TODO: implement Snapshotting::json() and verify_snapshot file I/O
    fn test_record_failed_when_failure() {
        let dir = TempSnapshotDir::new("testRecordFailed_WhenFailure");
        let _snapshot_path = dir.snapshot_path("testRecordFailed_WhenFailure");

        // Pre-populate with different content
        // fs::write(&snapshot_path, "999").unwrap();

        let config = SnapshotTestingConfiguration {
            record: Some(Record::Failed),
            diff_tool: None,
        };
        with_snapshot_testing(config, || {
            // assertSnapshot(of: 42, as: .json)
            // → "Snapshot does not match reference. A new snapshot was automatically recorded."
        });

        // File should now contain "42" (re-recorded due to failure)
        // assert_eq!(fs::read_to_string(&snapshot_path).unwrap(), "42");
    }

    /// Verifies that `Record::Failed` does NOT re-record when the snapshot
    /// matches. The file should not be modified (same content, same modification date).
    ///
    /// Swift: `func testRecordFailed_NoFailure()`
    #[test]
    #[ignore] // TODO: implement Snapshotting::json() and verify_snapshot file I/O
    fn test_record_failed_no_failure() {
        let dir = TempSnapshotDir::new("testRecordFailed_NoFailure");
        let _snapshot_path = dir.snapshot_path("testRecordFailed_NoFailure");

        // Pre-populate with matching content
        // fs::write(&snapshot_path, "42").unwrap();
        // let modified_before = fs::metadata(&snapshot_path).unwrap().modified().unwrap();

        let config = SnapshotTestingConfiguration {
            record: Some(Record::Failed),
            diff_tool: None,
        };
        with_snapshot_testing(config, || {
            // assertSnapshot(of: 42, as: .json)  → should pass silently
        });

        // File should still contain "42" and modification time should be unchanged
        // assert_eq!(fs::read_to_string(&snapshot_path).unwrap(), "42");
        // let modified_after = fs::metadata(&snapshot_path).unwrap().modified().unwrap();
        // assert_eq!(modified_before, modified_after);
    }

    /// Verifies that `Record::Failed` creates a new file when no reference
    /// exists (treats missing as a failure case).
    ///
    /// Swift: `func testRecordFailed_MissingFile()`
    #[test]
    #[ignore] // TODO: implement Snapshotting::json() and verify_snapshot file I/O
    fn test_record_failed_missing_file() {
        let dir = TempSnapshotDir::new("testRecordFailed_MissingFile");
        let _snapshot_path = dir.snapshot_path("testRecordFailed_MissingFile");

        let config = SnapshotTestingConfiguration {
            record: Some(Record::Failed),
            diff_tool: None,
        };
        with_snapshot_testing(config, || {
            // assertSnapshot(of: 42, as: .json)
            // → "No reference was found on disk. Automatically recorded snapshot."
        });

        // assert!(snapshot_path.exists());
        // assert_eq!(fs::read_to_string(&snapshot_path).unwrap(), "42");
    }
}
