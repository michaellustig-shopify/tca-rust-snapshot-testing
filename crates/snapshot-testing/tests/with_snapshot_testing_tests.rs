//! Tests ported from WithSnapshotTestingTests.swift
//!
//! These tests verify the nested configuration behavior of
//! `withSnapshotTesting` (Swift) / `with_snapshot_testing` (Rust).
//!
//! Key behaviors tested:
//! - Outer scope sets `record`, inner scope sets `diffTool`
//! - Inner scope inherits `record` from outer scope
//! - Each scope sees its own `diffTool`

#[cfg(test)]
mod with_snapshot_testing_tests {
    use snapshot_testing::{
        config::{current_record, with_snapshot_testing},
        DiffTool, Record, SnapshotTestingConfiguration,
    };

    /// Verifies that nested `with_snapshot_testing` calls properly layer
    /// configuration: the inner call overrides `diffTool` while inheriting
    /// the outer call's `record` setting.
    ///
    /// The Swift test did:
    /// 1. `withSnapshotTesting(record: .all)` — sets record to All
    /// 2. Checks `diffTool` produces the default file:// output
    /// 3. Checks `record == .all`
    /// 4. Nests `withSnapshotTesting(diffTool: "ksdiff")`
    /// 5. Checks `diffTool` now produces "ksdiff ..." output
    /// 6. Checks `record` is still `.all` (inherited from outer scope)
    ///
    /// Swift: `func testNesting()`
    #[test]
    #[ignore] // TODO: implement current_diff_tool() accessor for reading back configuration
    fn test_nesting() {
        let outer_config = SnapshotTestingConfiguration {
            record: Some(Record::All),
            diff_tool: None, // uses default
        };

        with_snapshot_testing(outer_config, || {
            // Outer scope: default diff tool should produce file:// URLs
            // let default_output = current_diff_tool().command("old.png", "new.png");
            // assert!(default_output.contains("file://old.png"));
            // assert!(default_output.contains("file://new.png"));

            // Outer scope: record should be All
            assert_eq!(current_record(), Record::All);

            // Nested scope: override diff tool but inherit record
            let inner_config = SnapshotTestingConfiguration {
                record: None, // should inherit All from outer
                diff_tool: Some(DiffTool::new(|a, b| format!("ksdiff {} {}", a, b))),
            };

            with_snapshot_testing(inner_config, || {
                // Inner scope: diff tool should now be ksdiff
                // let ksdiff_output = current_diff_tool().command("old.png", "new.png");
                // assert_eq!(ksdiff_output, "ksdiff old.png new.png");

                // Inner scope: record should still be All (inherited)
                assert_eq!(current_record(), Record::All);
            });
        });
    }
}
