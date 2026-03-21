//! Tests ported from SnapshotsTraitTests.swift
//!
//! In Swift, the `.snapshots(...)` trait allows configuring `record` and
//! `diffTool` at the suite/test level. These tests verify that:
//! - A diff tool set via trait produces the expected shell command
//! - Nested suites properly override or inherit configuration
//! - Record mode and diff tool can be set independently at different levels
//!
//! In Rust, the equivalent is `with_snapshot_testing` nesting.

#[cfg(test)]
mod snapshots_trait_tests {
    use snapshot_testing::{
        config::with_snapshot_testing, DiffTool, Record, SnapshotTestingConfiguration,
    };

    /// Verifies that a diff tool configured at the test level produces
    /// the correct command string.
    ///
    /// Swift equivalent: `.snapshots(diffTool: "ksdiff")` then
    /// `_diffTool(currentFilePath: "old.png", failedFilePath: "new.png") == "ksdiff old.png new.png"`
    ///
    /// Swift: `SnapshotsTraitTests.testDiffTool()`
    #[test]
    #[ignore] // TODO: implement _diff_tool() configuration accessor
    fn test_diff_tool() {
        let config = SnapshotTestingConfiguration {
            record: None,
            diff_tool: Some(DiffTool::new(|a, b| format!("ksdiff {} {}", a, b))),
        };
        with_snapshot_testing(config, || {
            // TODO: need a current_diff_tool() accessor to verify
            // let cmd = current_diff_tool().command("old.png", "new.png");
            // assert_eq!(cmd, "ksdiff old.png new.png");
        });
    }

    /// Verifies that a nested suite can override the diff tool from a parent suite.
    ///
    /// Parent sets `diffTool: "ksdiff"`, child overrides with `diffTool: "difftool"`.
    /// The child's test should see "difftool", not "ksdiff".
    ///
    /// Swift: `SnapshotsTraitTests.OverrideDiffTool.testDiffToolOverride()`
    #[test]
    #[ignore] // TODO: implement _diff_tool() configuration accessor
    fn test_diff_tool_override() {
        let outer_config = SnapshotTestingConfiguration {
            record: None,
            diff_tool: Some(DiffTool::new(|a, b| format!("ksdiff {} {}", a, b))),
        };
        with_snapshot_testing(outer_config, || {
            let inner_config = SnapshotTestingConfiguration {
                record: None,
                diff_tool: Some(DiffTool::new(|a, b| format!("difftool {} {}", a, b))),
            };
            with_snapshot_testing(inner_config, || {
                // TODO: need a current_diff_tool() accessor to verify
                // let cmd = current_diff_tool().command("old.png", "new.png");
                // assert_eq!(cmd, "difftool old.png new.png");
            });
        });
    }

    /// Verifies that overriding `record` in a nested suite preserves the
    /// parent's `diffTool`.
    ///
    /// Parent: `diffTool: "ksdiff"`, child: `record: .all`.
    /// The child should see `diffTool = "ksdiff"` (inherited) and `record = .all` (overridden).
    ///
    /// Swift: `SnapshotsTraitTests.OverrideDiffTool.OverrideRecord.config()`
    #[test]
    #[ignore] // TODO: implement _diff_tool() and _record() configuration accessors
    fn test_override_record_preserves_diff_tool() {
        let outer_config = SnapshotTestingConfiguration {
            record: None,
            diff_tool: Some(DiffTool::new(|a, b| format!("ksdiff {} {}", a, b))),
        };
        with_snapshot_testing(outer_config, || {
            let inner_config = SnapshotTestingConfiguration {
                record: Some(Record::All),
                diff_tool: None, // should inherit parent's ksdiff
            };
            with_snapshot_testing(inner_config, || {
                // TODO: need current_diff_tool() and current_record() accessors
                // let cmd = current_diff_tool().command("old.png", "new.png");
                // assert_eq!(cmd, "ksdiff old.png new.png");
                // assert_eq!(current_record(), Record::All);
            });
        });
    }

    /// Verifies that a deeply nested suite can override both `record` and `diffTool`
    /// simultaneously.
    ///
    /// Grandparent: `diffTool: "ksdiff"`, parent: `record: .all`,
    /// child: `record: .failed, diffTool: "diff"`.
    ///
    /// Swift: `SnapshotsTraitTests.OverrideDiffTool.OverrideRecord.OverrideDiffToolAndRecord.config()`
    #[test]
    #[ignore] // TODO: implement _diff_tool() and _record() configuration accessors
    fn test_override_diff_tool_and_record() {
        let level1 = SnapshotTestingConfiguration {
            record: None,
            diff_tool: Some(DiffTool::new(|a, b| format!("ksdiff {} {}", a, b))),
        };
        with_snapshot_testing(level1, || {
            let level2 = SnapshotTestingConfiguration {
                record: Some(Record::All),
                diff_tool: None,
            };
            with_snapshot_testing(level2, || {
                let level3 = SnapshotTestingConfiguration {
                    record: Some(Record::Failed),
                    diff_tool: Some(DiffTool::new(|a, b| format!("diff {} {}", a, b))),
                };
                with_snapshot_testing(level3, || {
                    // TODO: need current_diff_tool() and current_record() accessors
                    // let cmd = current_diff_tool().command("old.png", "new.png");
                    // assert_eq!(cmd, "diff old.png new.png");
                    // assert_eq!(current_record(), Record::Failed);
                });
            });
        });
    }
}
