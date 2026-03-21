//! ┌─────────────────────────────────────────────────────┐
//! │  ASSERT & VERIFY                                     │
//! │  Main assertion functions for snapshot testing       │
//! ├─────────────────────────────────────────────────────┤
//! │                                                      │
//! │  assert_snapshot(value, strategy)                     │
//! │       │                                              │
//! │       ▼                                              │
//! │  verify_snapshot(value, strategy)                     │
//! │       │                                              │
//! │       ├──► snapshot = strategy.snapshot(value)        │
//! │       │                                              │
//! │       ├──► load reference from __snapshots__/        │
//! │       │    └── if missing: record or fail            │
//! │       │                                              │
//! │       ├──► diff = strategy.diffing.diff(ref, snap)   │
//! │       │    └── if None: pass!                        │
//! │       │    └── if Some: fail with diff message       │
//! │       │                                              │
//! │       └──► return Ok(()) or Err(message)             │
//! │                                                      │
//! ├─────────────────────────────────────────────────────┤
//! │  WHY: assert_snapshot is the user-facing API.        │
//! │  verify_snapshot is the testable core that returns   │
//! │  a Result instead of panicking.                      │
//! │                                                      │
//! │  SWIFT EQUIVALENT: AssertSnapshot.swift               │
//! │  (Sources/SnapshotTesting/AssertSnapshot.swift)       │
//! │                                                      │
//! │  TESTED BY: tests/assert_tests.rs                    │
//! │  EDGE CASES: missing snapshot dir, record modes,     │
//! │  concurrent snapshot writes, path sanitization        │
//! │                                                      │
//! │  CHANGELOG:                                          │
//! │  • v0.1.0 — Initial stub with file I/O               │
//! │                                                      │
//! │  HISTORY: git log --oneline --follow -- crates/snapshot-testing/src/assert.rs
//! └─────────────────────────────────────────────────────┘

use crate::config::{current_record, Record};
use crate::snapshotting::Snapshotting;
use std::path::{Path, PathBuf};

/// Errors that can occur during snapshot verification.
#[derive(Debug, thiserror::Error)]
pub enum SnapshotError {
    /// The snapshot didn't match the reference and recording is disabled.
    #[error("Snapshot mismatch:\n{diff}")]
    Mismatch { diff: String },

    /// No reference snapshot exists and recording is disabled.
    #[error("No reference snapshot at {path}. Run with SNAPSHOT_TESTING_RECORD=all to record.")]
    MissingSnapshot { path: PathBuf },

    /// I/O error reading or writing snapshots.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The snapshot function timed out.
    #[error("Snapshot timed out after {seconds}s")]
    Timeout { seconds: f64 },
}

/// Verify a snapshot without panicking. Returns `Ok(())` if the snapshot
/// matches, or `Err(SnapshotError)` describing the mismatch.
///
/// This is the core logic. `assert_snapshot` wraps this and panics on error.
///
/// # Arguments
///
/// - `value`: The value to snapshot
/// - `snapshotting`: The strategy to use
/// - `name`: Optional named snapshot (uses a counter if None)
/// - `snapshot_dir`: Directory to store snapshots (typically `__snapshots__/<test_file>/`)
/// - `test_name`: Name of the test function (used for file naming)
///
/// # Swift Equivalent
///
/// ```swift
/// func verifySnapshot<Value, Format>(
///     of value: Value,
///     as snapshotting: Snapshotting<Value, Format>,
///     named name: String?,
///     ...
/// ) -> String?
/// ```
pub async fn verify_snapshot<V, F>(
    value: &V,
    snapshotting: &Snapshotting<V, F>,
    name: Option<&str>,
    snapshot_dir: &Path,
    test_name: &str,
) -> Result<(), SnapshotError>
where
    F: Clone,
{
    // 1. Generate the snapshot
    let snapshot = (snapshotting.snapshot)(value).await;

    // 2. Determine the snapshot file path
    let ext = snapshotting
        .path_extension
        .as_deref()
        .unwrap_or("txt");
    let file_name = match name {
        Some(n) => format!("{test_name}.{n}.{ext}"),
        None => format!("{test_name}.{ext}"),
    };
    let snapshot_path = snapshot_dir.join(&file_name);

    // 3. Load reference (if it exists)
    let record = current_record();
    let reference = if snapshot_path.exists() {
        let data = std::fs::read(&snapshot_path)?;
        Some((snapshotting.diffing.from_data)(&data))
    } else {
        None
    };

    match (reference, record) {
        // Reference exists — compare
        (Some(ref reference), _) => {
            if let Some((diff_msg, _attachments)) = (snapshotting.diffing.diff)(reference, &snapshot) {
                match record {
                    Record::All | Record::Failed => {
                        // Re-record
                        std::fs::create_dir_all(snapshot_dir)?;
                        let data = (snapshotting.diffing.to_data)(&snapshot);
                        std::fs::write(&snapshot_path, data)?;
                        Ok(())
                    }
                    _ => Err(SnapshotError::Mismatch { diff: diff_msg }),
                }
            } else {
                // Match!
                if record == Record::All {
                    // Still re-record in All mode
                    std::fs::create_dir_all(snapshot_dir)?;
                    let data = (snapshotting.diffing.to_data)(&snapshot);
                    std::fs::write(&snapshot_path, data)?;
                }
                Ok(())
            }
        }

        // No reference — record or fail
        (None, Record::Never) => Err(SnapshotError::MissingSnapshot {
            path: snapshot_path,
        }),

        (None, _) => {
            // Record the snapshot
            std::fs::create_dir_all(snapshot_dir)?;
            let data = (snapshotting.diffing.to_data)(&snapshot);
            std::fs::write(&snapshot_path, data)?;
            Ok(())
        }
    }
}

/// Assert that a snapshot matches the reference. Panics on mismatch.
///
/// This is the primary user-facing API for snapshot testing.
///
/// # Panics
///
/// Panics with a descriptive message if the snapshot doesn't match
/// or if a reference is missing (in Never mode).
///
/// # Swift Equivalent
///
/// ```swift
/// func assertSnapshot<Value, Format>(
///     of value: @autoclosure () throws -> Value,
///     as snapshotting: Snapshotting<Value, Format>,
///     ...
/// )
/// ```
pub async fn assert_snapshot<V, F>(
    value: &V,
    snapshotting: &Snapshotting<V, F>,
    name: Option<&str>,
    snapshot_dir: &Path,
    test_name: &str,
) where
    F: Clone,
{
    if let Err(e) = verify_snapshot(value, snapshotting, name, snapshot_dir, test_name).await {
        panic!("{e}");
    }
}
