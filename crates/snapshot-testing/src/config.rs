//! ┌─────────────────────────────────────────────────────┐
//! │  CONFIGURATION                                       │
//! │  Runtime settings for snapshot testing behavior      │
//! ├─────────────────────────────────────────────────────┤
//! │                                                      │
//! │  SnapshotTestingConfiguration                        │
//! │  ├── record: Record                                  │
//! │  │   ├── All       (always write snapshots)          │
//! │  │   ├── Failed    (write only on mismatch)          │
//! │  │   ├── Missing   (write only if file absent)       │
//! │  │   └── Never     (CI mode, never write)            │
//! │  └── diff_tool: DiffTool                             │
//! │      └── command(current, failed) → String           │
//! │                                                      │
//! ├─────────────────────────────────────────────────────┤
//! │  WHY: Configuration must be injectable at runtime    │
//! │  so tests can run in different modes (record vs.     │
//! │  verify, CI vs. local) without recompilation.        │
//! │                                                      │
//! │  SWIFT EQUIVALENT: SnapshotTestingConfiguration      │
//! │  (Sources/SnapshotTesting/SnapshotTestingConfiguration.swift)
//! │                                                      │
//! │  TESTED BY: tests/config_tests.rs                    │
//! │  EDGE CASES: env var override, nested withConfig     │
//! │                                                      │
//! │  CHANGELOG:                                          │
//! │  • v0.1.0 — Initial Record, DiffTool, Config types   │
//! │                                                      │
//! │  HISTORY: git log --oneline --follow -- crates/snapshot-testing/src/config.rs
//! └─────────────────────────────────────────────────────┘

use std::cell::RefCell;
use std::sync::Arc;

/// Controls when snapshots are written to disk.
///
/// # Swift Equivalent
///
/// ```swift
/// enum Record { case all, failed, missing, never }
/// ```
///
/// # Environment Variable
///
/// Can be overridden via `SNAPSHOT_TESTING_RECORD=all|missing|failed|never`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Record {
    /// Always write snapshots to disk, even if they match.
    /// Use during initial development to (re)generate all snapshots.
    All,

    /// Write only when the comparison fails.
    /// Useful for fuzzy-matching strategies where you want to update
    /// on regression but not on every run.
    Failed,

    /// Write only if the snapshot file doesn't exist yet.
    /// Default mode — new tests get recorded, existing ones are verified.
    Missing,

    /// Never write to disk. Fails if snapshot is missing.
    /// Use in CI to ensure all snapshots are committed.
    Never,
}

impl Record {
    /// Parse from the `SNAPSHOT_TESTING_RECORD` environment variable.
    ///
    /// Returns `None` if the env var is not set.
    /// Panics if set to an unrecognized value (fail-fast in tests).
    pub fn from_env() -> Option<Self> {
        std::env::var("SNAPSHOT_TESTING_RECORD").ok().map(|s| match s.as_str() {
            "all" => Record::All,
            "failed" => Record::Failed,
            "missing" => Record::Missing,
            "never" => Record::Never,
            other => panic!(
                "Invalid SNAPSHOT_TESTING_RECORD value: '{other}'. Expected: all, failed, missing, never"
            ),
        })
    }
}

/// A tool for viewing diffs between expected and actual snapshots.
///
/// Given two file paths, produces a shell command string that opens
/// a visual diff tool.
///
/// # Swift Equivalent
///
/// ```swift
/// struct DiffTool {
///     var command: (String, String) -> String
/// }
/// ```
#[derive(Clone)]
pub struct DiffTool {
    /// Function that takes (current_file_path, failed_file_path) and
    /// returns a shell command to view the diff.
    command: Arc<dyn Fn(&str, &str) -> String + Send + Sync>,
}

impl DiffTool {
    /// Create a custom diff tool.
    ///
    /// # Examples
    ///
    /// ```
    /// use snapshot_testing::config::DiffTool;
    /// let tool = DiffTool::new(|a, b| format!("diff {a} {b}"));
    /// assert!(tool.command("foo.txt", "bar.txt").contains("diff"));
    /// ```
    pub fn new<F>(command: F) -> Self
    where
        F: Fn(&str, &str) -> String + Send + Sync + 'static,
    {
        DiffTool {
            command: Arc::new(command),
        }
    }

    /// Default diff tool — prints file:// URLs for easy clicking in terminals.
    ///
    /// # Examples
    ///
    /// ```
    /// use snapshot_testing::config::DiffTool;
    /// let tool = DiffTool::default_tool();
    /// let output = tool.command("/tmp/expected.txt", "/tmp/actual.txt");
    /// assert!(output.contains("file://"));
    /// ```
    pub fn default_tool() -> Self {
        DiffTool::new(|current, failed| {
            format!("Snapshot diff:\n  Expected: file://{current}\n  Actual:   file://{failed}")
        })
    }

    /// Generate the diff command for two file paths.
    pub fn command(&self, current: &str, failed: &str) -> String {
        (self.command)(current, failed)
    }
}

impl std::fmt::Debug for DiffTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiffTool").finish_non_exhaustive()
    }
}

/// Runtime configuration for snapshot testing.
///
/// Use `with_snapshot_testing` to set configuration for a scope.
///
/// # Swift Equivalent
///
/// ```swift
/// struct SnapshotTestingConfiguration {
///     var record: Record?
///     var diffTool: DiffTool?
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SnapshotTestingConfiguration {
    /// Recording mode. `None` means use the default (Missing).
    pub record: Option<Record>,

    /// Diff tool for viewing mismatches. `None` means use the default.
    pub diff_tool: Option<DiffTool>,
}

impl Default for SnapshotTestingConfiguration {
    fn default() -> Self {
        SnapshotTestingConfiguration {
            record: None,
            diff_tool: None,
        }
    }
}

// Thread-local configuration stack for `with_snapshot_testing` scoping
thread_local! {
    static CONFIG_STACK: RefCell<Vec<SnapshotTestingConfiguration>> = RefCell::new(Vec::new());
}

/// Run a closure with custom snapshot testing configuration.
///
/// Configuration is scoped — it applies only within the closure and
/// is automatically restored when the closure returns. Calls can be nested.
///
/// # Swift Equivalent
///
/// ```swift
/// func withSnapshotTesting<R>(
///     record: Record? = nil,
///     diffTool: DiffTool? = nil,
///     operation: () throws -> R
/// ) rethrows -> R
/// ```
///
/// # Why thread_local instead of a global?
///
/// Tests run in parallel across threads. A global config would cause
/// races. Thread-local storage gives each test thread its own config
/// stack, matching Swift's task-local approach.
pub fn with_snapshot_testing<R, F: FnOnce() -> R>(config: SnapshotTestingConfiguration, f: F) -> R {
    CONFIG_STACK.with(|stack| {
        stack.borrow_mut().push(config);
    });
    let result = f();
    CONFIG_STACK.with(|stack| {
        stack.borrow_mut().pop();
    });
    result
}

/// Get the current effective record mode.
///
/// Priority: thread-local config > env var > default (Missing)
pub fn current_record() -> Record {
    CONFIG_STACK.with(|stack| {
        let stack = stack.borrow();
        for config in stack.iter().rev() {
            if let Some(record) = config.record {
                return record;
            }
        }
        Record::from_env().unwrap_or(Record::Missing)
    })
}
