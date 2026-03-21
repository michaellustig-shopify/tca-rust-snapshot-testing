// ============================================================================
// Exercise 04: Record Modes — Configuration and Error Handling
// Lessons: docs/curriculum/05-error-handling.md, docs/curriculum/10-advanced.md
//
// Run with: cargo run -p snapshot-testing --example ex04_record_modes
//
// This exercise teaches you:
//   1. How Record modes control snapshot behavior
//   2. Thread-local configuration with with_snapshot_testing
//   3. Nesting configurations (inner overrides outer)
//   4. Result<T, E> and pattern matching
//   5. The RefCell borrow pattern inside thread_local!
// ============================================================================

use snapshot_testing::{
    Record, SnapshotTestingConfiguration, DiffTool,
    config::{with_snapshot_testing, current_record},
};

fn main() {
    println!("=== Exercise 04: Record Modes ===\n");

    // -----------------------------------------------------------------------
    // Part 1: The four Record modes
    // -----------------------------------------------------------------------

    println!("--- Part 1: Record Modes ---\n");

    let modes = [
        (Record::All, "All", "Always write snapshots, even if they match"),
        (Record::Failed, "Failed", "Write only when comparison fails"),
        (Record::Missing, "Missing", "Write only if snapshot file doesn't exist"),
        (Record::Never, "Never", "Never write — fail if missing (CI mode)"),
    ];

    for (mode, name, desc) in &modes {
        println!("  Record::{:<10} — {}", name, desc);
        // Record implements Copy, so we can use it without cloning
        let _copy = *mode;  // Copy trait makes this work!
        // Record implements PartialEq, so we can compare
        assert_eq!(*mode, _copy);
    }
    println!();

    // -----------------------------------------------------------------------
    // Part 2: Default record mode (when nothing is configured)
    // -----------------------------------------------------------------------

    println!("--- Part 2: Default Record Mode ---\n");

    let default_record = current_record();
    println!("Default record mode: {:?}", default_record);
    println!("(This is Record::Missing unless SNAPSHOT_TESTING_RECORD env var is set)");
    println!();

    // -----------------------------------------------------------------------
    // Part 3: Setting record mode with with_snapshot_testing
    // -----------------------------------------------------------------------
    // with_snapshot_testing pushes a config onto a thread-local stack.
    // Inside the closure, current_record() reads from that stack.
    // When the closure returns, the config is popped.

    println!("--- Part 3: Setting Record Mode ---\n");

    let config = SnapshotTestingConfiguration {
        record: Some(Record::All),
        diff_tool: None,
    };

    println!("Before with_snapshot_testing: {:?}", current_record());

    with_snapshot_testing(config, || {
        println!("Inside with_snapshot_testing: {:?}", current_record());
        assert_eq!(current_record(), Record::All);
    });

    println!("After with_snapshot_testing:  {:?}", current_record());
    println!("(Restored to default — the config was popped from the stack)");
    println!();

    // -----------------------------------------------------------------------
    // Part 4: Nesting configurations
    // -----------------------------------------------------------------------
    // Inner scopes override outer scopes. But only for the fields they set.
    // If the inner scope sets record=None, it inherits from the outer scope.

    println!("--- Part 4: Nesting Configurations ---\n");

    let outer_config = SnapshotTestingConfiguration {
        record: Some(Record::All),
        diff_tool: Some(DiffTool::new(|a, b| format!("ksdiff {} {}", a, b))),
    };

    with_snapshot_testing(outer_config, || {
        println!("Outer scope: record = {:?}", current_record());
        assert_eq!(current_record(), Record::All);

        // Inner scope: override record but don't set diff_tool
        let inner_config = SnapshotTestingConfiguration {
            record: Some(Record::Never),
            diff_tool: None,  // inherits from outer scope
        };

        with_snapshot_testing(inner_config, || {
            println!("Inner scope: record = {:?}", current_record());
            assert_eq!(current_record(), Record::Never);
        });

        // After inner scope: back to outer's Record::All
        println!("Back to outer: record = {:?}", current_record());
        assert_eq!(current_record(), Record::All);
    });

    println!("Outside all scopes: record = {:?}", current_record());
    println!();

    // -----------------------------------------------------------------------
    // Part 5: Three levels deep
    // -----------------------------------------------------------------------
    // This mirrors the SnapshotsTraitTests from the test suite.

    println!("--- Part 5: Three Levels Deep ---\n");

    let level1 = SnapshotTestingConfiguration {
        record: Some(Record::Missing),
        diff_tool: None,
    };

    with_snapshot_testing(level1, || {
        println!("Level 1: {:?}", current_record());

        let level2 = SnapshotTestingConfiguration {
            record: Some(Record::All),
            diff_tool: None,
        };

        with_snapshot_testing(level2, || {
            println!("Level 2: {:?}", current_record());

            let level3 = SnapshotTestingConfiguration {
                record: Some(Record::Failed),
                diff_tool: None,
            };

            with_snapshot_testing(level3, || {
                println!("Level 3: {:?}", current_record());
                assert_eq!(current_record(), Record::Failed);
            });

            // Back to level 2
            assert_eq!(current_record(), Record::All);
        });

        // Back to level 1
        assert_eq!(current_record(), Record::Missing);
    });

    println!("All scopes exited: {:?}", current_record());
    println!();

    // -----------------------------------------------------------------------
    // Part 6: Result<T, E> pattern matching
    // -----------------------------------------------------------------------
    // verify_snapshot returns Result<(), SnapshotError>.
    // Here we demonstrate the pattern with a simulated result.

    println!("--- Part 6: Result Pattern Matching ---\n");

    // Simulating what verify_snapshot might return
    let success: Result<(), String> = Ok(());
    let mismatch: Result<(), String> = Err("Snapshot mismatch: -hello +goodbye".into());
    let missing: Result<(), String> = Err("No reference snapshot at /tmp/test.txt".into());

    for (i, result) in [success, mismatch, missing].iter().enumerate() {
        match result {
            Ok(()) => println!("  Result {}: PASS", i + 1),
            Err(msg) => println!("  Result {}: FAIL — {}", i + 1, msg),
        }
    }
    println!();

    // The ? operator: propagates errors early
    fn might_fail(should_fail: bool) -> Result<String, String> {
        let step1 = if should_fail {
            Err("step 1 failed".to_string())
        } else {
            Ok("step 1 ok".to_string())
        };

        let result = step1?;  // Returns Err early if step1 failed
        Ok(format!("{} -> step 2 ok", result))
    }

    println!("  might_fail(false) = {:?}", might_fail(false));
    println!("  might_fail(true)  = {:?}", might_fail(true));
    println!();

    // -----------------------------------------------------------------------
    // Part 7: YOUR TURN — Create a nested config scenario
    // -----------------------------------------------------------------------

    println!("--- Part 7: Your Turn ---\n");
    println!("TODO: Create a 3-level nesting where:");
    println!("  Level 1: Record::Never");
    println!("  Level 2: Record::Missing (overrides level 1)");
    println!("  Level 3: doesn't set record (should inherit Record::Missing from level 2)");
    println!("Verify each level with assert_eq!(current_record(), expected)");

    // Example solution (uncomment to try):
    //
    // with_snapshot_testing(SnapshotTestingConfiguration {
    //     record: Some(Record::Never), diff_tool: None,
    // }, || {
    //     assert_eq!(current_record(), Record::Never);
    //     with_snapshot_testing(SnapshotTestingConfiguration {
    //         record: Some(Record::Missing), diff_tool: None,
    //     }, || {
    //         assert_eq!(current_record(), Record::Missing);
    //         with_snapshot_testing(SnapshotTestingConfiguration {
    //             record: None, diff_tool: None,  // inherits Missing
    //         }, || {
    //             assert_eq!(current_record(), Record::Missing);
    //             println!("  All assertions passed!");
    //         });
    //     });
    // });

    println!("\n=== Exercise 04 Complete ===");
}
