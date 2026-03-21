//! Test helpers ported from Swift's Internal/TestHelpers.swift, BaseSuite.swift, BaseTestCase.swift
//!
//! In the Swift version, `BaseTestCase` wraps every test invocation with
//! `withSnapshotTesting(record: .failed, diffTool: .ksdiff)`. In Rust we
//! provide a helper function that does the same thing with
//! `with_snapshot_testing`.

#[allow(unused_imports)]
use snapshot_testing::{
    DiffTool, Record, SnapshotTestingConfiguration,
    config::with_snapshot_testing,
};

// -----------------------------------------------------------------------
// Constants (ported from TestHelpers.swift platform detection)
// -----------------------------------------------------------------------

/// Platform identifier string.
///
/// In the Swift tests this was a compile-time constant set by `#if os(iOS)`.
/// In Rust we always report the host platform. Tests that were iOS/tvOS/macOS
/// specific are marked `#[ignore]` since they exercise Apple-only UI types.
#[allow(dead_code)]
pub const PLATFORM: &str = if cfg!(target_os = "macos") {
    "macos"
} else if cfg!(target_os = "linux") {
    "linux"
} else {
    "unknown"
};

// -----------------------------------------------------------------------
// Base configuration (ported from BaseTestCase.swift / BaseSuite.swift)
// -----------------------------------------------------------------------

/// Run `f` with the same default configuration that the Swift `BaseTestCase`
/// applied to every test: `record: .failed`, `diffTool: .ksdiff`.
///
/// # Examples
///
/// ```rust
/// # // This is a doc-test showing usage — the function is test-only.
/// # fn main() {
/// // In a real test:
/// // with_base_config(|| {
/// //     assert_snapshot!(...);
/// // });
/// # }
/// ```
#[allow(dead_code)]
pub fn with_base_config<R>(f: impl FnOnce() -> R) -> R {
    let config = SnapshotTestingConfiguration {
        record: Some(Record::Failed),
        diff_tool: Some(DiffTool::new(|a, b| format!("ksdiff {} {}", a, b))),
    };
    with_snapshot_testing(config, f)
}
