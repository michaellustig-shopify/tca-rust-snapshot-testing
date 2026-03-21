# Verification Matrix

## rust-snapshot-testing v1.0.0

Maps every SRS requirement to its test(s), doc comments, and current status.

---

## Table of Contents

1. [Coverage Summary](#1-coverage-summary)
2. [Core Types Verification](#2-core-types-verification)
3. [Assertion Functions Verification](#3-assertion-functions-verification)
4. [Configuration System Verification](#4-configuration-system-verification)
5. [Built-in Strategies Verification](#5-built-in-strategies-verification)
6. [Diff Algorithm Verification](#6-diff-algorithm-verification)
7. [Inline Snapshot Verification](#7-inline-snapshot-verification)
8. [Custom Dump Verification](#8-custom-dump-verification)
9. [File System Verification](#9-file-system-verification)
10. [Concurrency Verification](#10-concurrency-verification)
11. [Error Handling Verification](#11-error-handling-verification)
12. [Environment Variable Verification](#12-environment-variable-verification)
13. [Identified Gaps](#13-identified-gaps)

---

## 1. Coverage Summary

| Category | Total Requirements | Tested | Doc'd | Implemented | Gaps |
|----------|-------------------|--------|-------|-------------|------|
| Core Types | 12 | 0 | 5 | 6 | 6 |
| Assertions | 11 | 0 | 2 | 2 | 9 |
| Configuration | 8 | 0 | 4 | 4 | 4 |
| Strategies | 16 | 0 | 0 | 0 | 16 |
| Diff Algorithm | 5 | 0 | 2 | 2 | 3 |
| Inline Snapshots | 5 | 0 | 0 | 0 | 5 |
| Custom Dump | 3 | 0 | 0 | 0 | 3 |
| File System | 5 | 0 | 0 | 1 | 4 |
| Concurrency | 5 | 0 | 1 | 1 | 4 |
| Error Handling | 3 | 0 | 1 | 1 | 2 |
| Environment | 2 | 0 | 1 | 1 | 1 |
| **TOTAL** | **75** | **0** | **16** | **18** | **57** |

---

## 2. Core Types Verification

### 2.1 Snapshotting

| Req ID | Requirement | Test File | Test Function | Doc Comment | Status |
|--------|------------|-----------|---------------|-------------|--------|
| REQ-SNAP-001 | Snapshotting struct definition | `tests/snapshotting_tests.rs` | `test_snapshotting_new` | YES (snapshotting.rs) | PARTIAL - struct exists, missing `new_sync` |
| REQ-SNAP-001a | Async constructor | `tests/snapshotting_tests.rs` | `test_snapshotting_async_new` | NEEDED | PARTIAL - `new` exists |
| REQ-SNAP-001b | Sync constructor | `tests/snapshotting_tests.rs` | `test_snapshotting_sync_new` | NEEDED | NOT STARTED |
| REQ-SNAP-001c | Identity constructor (V==F) | `tests/snapshotting_tests.rs` | `test_snapshotting_identity` | NEEDED | NOT STARTED |
| REQ-SNAP-002 | pullback (sync) | `tests/snapshotting_tests.rs` | `test_pullback_basic` | YES | PARTIAL |
| REQ-SNAP-002-B | pullback preserves fields | `tests/snapshotting_tests.rs` | `test_pullback_preserves_extension` | NEEDED | NOT TESTED |
| REQ-SNAP-003 | async_pullback | `tests/snapshotting_tests.rs` | `test_async_pullback` | NEEDED | NOT STARTED |
| REQ-SNAP-003-B | async_pullback chains futures | `tests/snapshotting_tests.rs` | `test_async_pullback_chaining` | NEEDED | NOT STARTED |
| REQ-SNAP-004 | Clone impl | `tests/snapshotting_tests.rs` | `test_snapshotting_clone` | YES | DONE |
| REQ-SNAP-005 | SimplySnapshotting alias | `tests/snapshotting_tests.rs` | `test_simply_snapshotting` | NEEDED | NOT STARTED |
| REQ-ATTACH-001 | DiffAttachment enum | `tests/diffing_tests.rs` | `test_diff_attachment` | YES | DONE |

### Test Plan: Snapshotting

```rust
// tests/snapshotting_tests.rs

#[tokio::test]
async fn test_snapshotting_new() {
    // Create a Snapshotting<i32, String> that converts i32 to string
    // Verify snapshot function works
    // Verify path_extension is stored
}

#[tokio::test]
async fn test_snapshotting_sync_new() {
    // Create with new_sync (non-async snapshot fn)
    // Verify it wraps in ready() future
}

#[tokio::test]
async fn test_snapshotting_identity() {
    // Create Snapshotting<String, String>::identity()
    // Verify snapshot returns the value unchanged
}

#[tokio::test]
async fn test_pullback_basic() {
    // Start with Snapshotting<String, String>::lines()
    // Pullback through Display::to_string for i32
    // Verify: strategy.snapshot(&42) produces "42"
}

#[tokio::test]
async fn test_pullback_preserves_extension() {
    // Create strategy with extension "json"
    // Pullback to new type
    // Verify extension is still "json"
}

#[tokio::test]
async fn test_pullback_preserves_diffing() {
    // Create strategy with custom diffing
    // Pullback to new type
    // Verify diffing still works (to_data, from_data, diff)
}

#[tokio::test]
async fn test_pullback_composition() {
    // Chain: String strategy -> i32 strategy -> MyStruct strategy
    // Verify the whole chain works end-to-end
}

#[tokio::test]
async fn test_async_pullback() {
    // Pullback with an async transform
    // Verify the transform future is awaited before snapshot
}

#[test]
fn test_snapshotting_clone() {
    // Clone a snapshotting
    // Verify both original and clone work independently
}

#[test]
fn test_simply_snapshotting_alias() {
    // Create SimplySnapshotting<String>
    // Verify V == F
}
```

---

## 3. Assertion Functions Verification

### 3.1 assert_snapshot

| Req ID | Requirement | Test File | Test Function | Doc Comment | Status |
|--------|------------|-----------|---------------|-------------|--------|
| REQ-ASSERT-001 | assert_snapshot macro | `tests/assert_tests.rs` | `test_assert_snapshot_pass` | YES | PARTIAL (function, not macro) |
| REQ-ASSERT-001a | Captures file/line | `tests/assert_tests.rs` | `test_assert_captures_location` | NEEDED | NOT STARTED |
| REQ-ASSERT-001b | Derives snapshot dir | `tests/assert_tests.rs` | `test_assert_derives_dir` | NEEDED | NOT STARTED |
| REQ-ASSERT-001c | Derives test name | `tests/assert_tests.rs` | `test_assert_derives_test_name` | NEEDED | NOT STARTED |
| REQ-ASSERT-001d | Panics on mismatch | `tests/assert_tests.rs` | `test_assert_panics_on_mismatch` | NEEDED | NOT STARTED |
| REQ-ASSERT-001e | Timeout support | `tests/assert_tests.rs` | `test_assert_timeout` | NEEDED | NOT STARTED |

### 3.2 verify_snapshot

| Req ID | Requirement | Test File | Test Function | Doc Comment | Status |
|--------|------------|-----------|---------------|-------------|--------|
| REQ-VERIFY-001 | verify_snapshot function | `tests/assert_tests.rs` | `test_verify_snapshot_basic` | YES | PARTIAL |
| REQ-VERIFY-001a | Generates snapshot | `tests/assert_tests.rs` | `test_verify_generates_snapshot` | NEEDED | DONE |
| REQ-VERIFY-001b | Returns timeout error | `tests/assert_tests.rs` | `test_verify_timeout` | NEEDED | NOT STARTED |
| REQ-VERIFY-001c | Returns snapshot failed | `tests/assert_tests.rs` | `test_verify_snapshot_failed` | NEEDED | NOT STARTED |
| REQ-VERIFY-001d | Record::All writes to disk | `tests/assert_tests.rs` | `test_verify_record_all` | NEEDED | PARTIAL |
| REQ-VERIFY-001e | Missing + Never = error | `tests/assert_tests.rs` | `test_verify_missing_never` | NEEDED | DONE |
| REQ-VERIFY-001f | Missing + not-Never = record | `tests/assert_tests.rs` | `test_verify_missing_records` | NEEDED | DONE |
| REQ-VERIFY-001g | Match returns Ok | `tests/assert_tests.rs` | `test_verify_match_returns_ok` | NEEDED | DONE |
| REQ-VERIFY-001h | Mismatch + Failed = re-record | `tests/assert_tests.rs` | `test_verify_mismatch_failed_rerecords` | NEEDED | PARTIAL |
| REQ-VERIFY-001i | Mismatch + other = error | `tests/assert_tests.rs` | `test_verify_mismatch_error` | NEEDED | PARTIAL |

### 3.3 Snapshot File Naming

| Req ID | Requirement | Test File | Test Function | Doc Comment | Status |
|--------|------------|-----------|---------------|-------------|--------|
| REQ-VERIFY-002a | Default directory layout | `tests/assert_tests.rs` | `test_snapshot_directory_layout` | NEEDED | NOT TESTED |
| REQ-VERIFY-002b | Named snapshot file | `tests/assert_tests.rs` | `test_named_snapshot_file` | NEEDED | PARTIAL |
| REQ-VERIFY-002c | Unnamed first snapshot | `tests/assert_tests.rs` | `test_unnamed_first_snapshot` | NEEDED | NOT STARTED |
| REQ-VERIFY-002d | Counter incrementing | `tests/assert_tests.rs` | `test_snapshot_counter` | NEEDED | NOT STARTED |
| REQ-VERIFY-002e | Path sanitization | `tests/assert_tests.rs` | `test_path_sanitization` | NEEDED | NOT STARTED |

### Test Plan: Path Sanitization

```rust
#[test]
fn test_path_sanitization() {
    assert_eq!(sanitize_path_component("test_foo()"), "test_foo");
    assert_eq!(sanitize_path_component("test with spaces"), "test-with-spaces");
    assert_eq!(sanitize_path_component("test/with/slashes"), "test-with-slashes");
    assert_eq!(sanitize_path_component("---leading---"), "leading");
    assert_eq!(sanitize_path_component("unicode_test_\u{1F600}"), "unicode_test");
}
```

---

## 4. Configuration System Verification

| Req ID | Requirement | Test File | Test Function | Doc Comment | Status |
|--------|------------|-----------|---------------|-------------|--------|
| REQ-CONFIG-001 | Configuration struct | `tests/config_tests.rs` | `test_config_default` | YES | DONE |
| REQ-CONFIG-002 | Priority resolution | `tests/config_tests.rs` | `test_config_priority` | NEEDED | NOT TESTED |
| REQ-RECORD-001 | Record enum | `tests/config_tests.rs` | `test_record_variants` | YES | DONE |
| REQ-RECORD-001a | Parse from env var | `tests/config_tests.rs` | `test_record_from_env` | YES | DONE |
| REQ-RECORD-001b | Invalid env panics | `tests/config_tests.rs` | `test_record_invalid_env_panics` | NEEDED | DONE |
| REQ-RECORD-001c | Derive traits | `tests/config_tests.rs` | `test_record_traits` | NEEDED | DONE |
| REQ-DIFFTOOL-001a | Default diff tool | `tests/config_tests.rs` | `test_difftool_default` | YES | DONE |
| REQ-DIFFTOOL-001b | ksdiff preset | `tests/config_tests.rs` | `test_difftool_ksdiff` | NEEDED | NOT STARTED |
| REQ-DIFFTOOL-001c | Custom closure | `tests/config_tests.rs` | `test_difftool_custom` | NEEDED | DONE |
| REQ-DIFFTOOL-002 | DiffTool Clone | `tests/config_tests.rs` | `test_difftool_clone` | NEEDED | DONE |
| REQ-SCOPE-001 | with_snapshot_testing | `tests/config_tests.rs` | `test_with_snapshot_testing` | YES | DONE |
| REQ-SCOPE-001a | Scoped to closure | `tests/config_tests.rs` | `test_scope_restores_on_return` | NEEDED | NOT TESTED |
| REQ-SCOPE-001b | Nested configs | `tests/config_tests.rs` | `test_nested_configs` | NEEDED | NOT TESTED |
| REQ-SCOPE-001c | Thread-local | `tests/config_tests.rs` | `test_thread_local_isolation` | NEEDED | NOT TESTED |
| REQ-SCOPE-001d | Panic-safe restore | `tests/config_tests.rs` | `test_config_restored_on_panic` | NEEDED | NOT STARTED |

### Test Plan: Configuration Priority

```rust
#[test]
fn test_config_priority() {
    // 1. Default (Missing)
    assert_eq!(current_record(), Record::Missing);

    // 2. Env var overrides default
    std::env::set_var("SNAPSHOT_TESTING_RECORD", "never");
    assert_eq!(current_record(), Record::Never);

    // 3. Thread-local overrides env var
    with_snapshot_testing(
        SnapshotTestingConfiguration { record: Some(Record::All), ..Default::default() },
        || {
            assert_eq!(current_record(), Record::All);

            // 4. Nested overrides outer
            with_snapshot_testing(
                SnapshotTestingConfiguration { record: Some(Record::Failed), ..Default::default() },
                || {
                    assert_eq!(current_record(), Record::Failed);
                },
            );

            // 5. Restored after nested exits
            assert_eq!(current_record(), Record::All);
        },
    );

    // 6. Restored after outer exits
    assert_eq!(current_record(), Record::Never); // env var still set

    std::env::remove_var("SNAPSHOT_TESTING_RECORD");
}

#[test]
fn test_config_restored_on_panic() {
    let result = std::panic::catch_unwind(|| {
        with_snapshot_testing(
            SnapshotTestingConfiguration { record: Some(Record::All), ..Default::default() },
            || { panic!("test panic"); },
        );
    });
    assert!(result.is_err());
    // Config must be restored even after panic
    assert_eq!(current_record(), Record::Missing);
}
```

---

## 5. Built-in Strategies Verification

| Req ID | Requirement | Test File | Test Function | Doc Comment | Status |
|--------|------------|-----------|---------------|-------------|--------|
| REQ-STRAT-001 | lines strategy | `tests/strategy_tests.rs` | `test_lines_strategy` | NEEDED | NOT STARTED |
| REQ-STRAT-001a | path_extension = "txt" | `tests/strategy_tests.rs` | `test_lines_extension` | NEEDED | NOT STARTED |
| REQ-STRAT-001b | to_data = UTF-8 | `tests/strategy_tests.rs` | `test_lines_to_data` | NEEDED | NOT STARTED |
| REQ-STRAT-001c | from_data = UTF-8 | `tests/strategy_tests.rs` | `test_lines_from_data` | NEEDED | NOT STARTED |
| REQ-STRAT-001d | diff = unified diff | `tests/strategy_tests.rs` | `test_lines_diff` | NEEDED | NOT STARTED |
| REQ-STRAT-002 | Diffing::lines | `tests/strategy_tests.rs` | `test_diffing_lines` | NEEDED | NOT STARTED |
| REQ-STRAT-002a | None when identical | `tests/strategy_tests.rs` | `test_diffing_lines_identical` | NEEDED | NOT STARTED |
| REQ-STRAT-002b | Hunks with @@ markers | `tests/strategy_tests.rs` | `test_diffing_lines_hunks` | NEEDED | NOT STARTED |
| REQ-STRAT-002c | Patch attachment | `tests/strategy_tests.rs` | `test_diffing_lines_attachment` | NEEDED | NOT STARTED |
| REQ-STRAT-003 | description strategy | `tests/strategy_tests.rs` | `test_description_strategy` | NEEDED | NOT STARTED |
| REQ-STRAT-004 | debug strategy | `tests/strategy_tests.rs` | `test_debug_strategy` | NEEDED | NOT STARTED |
| REQ-STRAT-005 | json strategy | `tests/strategy_tests.rs` | `test_json_strategy` | NEEDED | NOT STARTED |
| REQ-STRAT-005a | json path_extension | `tests/strategy_tests.rs` | `test_json_extension` | NEEDED | NOT STARTED |
| REQ-STRAT-005b | json pretty + sorted | `tests/strategy_tests.rs` | `test_json_sorted_keys` | NEEDED | NOT STARTED |
| REQ-STRAT-006 | data strategy | `tests/strategy_tests.rs` | `test_data_strategy` | NEEDED | NOT STARTED |
| REQ-STRAT-007 | func strategy | `tests/strategy_tests.rs` | `test_func_strategy` | NEEDED | NOT STARTED |
| REQ-STRAT-008 | HTTP request strategy | `tests/strategy_tests.rs` | `test_raw_request` | NEEDED | NOT STARTED |
| REQ-STRAT-008c | curl strategy | `tests/strategy_tests.rs` | `test_curl_request` | NEEDED | NOT STARTED |
| REQ-STRAT-009 | wait strategy | `tests/strategy_tests.rs` | `test_wait_strategy` | NEEDED | NOT STARTED |

### Test Plan: Lines Strategy

```rust
#[tokio::test]
async fn test_lines_strategy() {
    let strat = Snapshotting::<String, String>::lines();
    assert_eq!(strat.path_extension.as_deref(), Some("txt"));

    // Snapshot is identity for String -> String
    let result = (strat.snapshot)(&"hello\nworld".to_string()).await;
    assert_eq!(result, "hello\nworld");

    // Diff: identical strings -> None
    let diff_result = (strat.diffing.diff)(&"hello".to_string(), &"hello".to_string());
    assert!(diff_result.is_none());

    // Diff: different strings -> Some with unified diff
    let diff_result = (strat.diffing.diff)(&"hello".to_string(), &"world".to_string());
    assert!(diff_result.is_some());
    let (message, attachments) = diff_result.unwrap();
    assert!(message.contains("@@"));
    assert_eq!(attachments.len(), 1); // difference.patch
}

#[tokio::test]
async fn test_json_strategy() {
    #[derive(serde::Serialize)]
    struct User { name: String, age: u32 }

    let strat = Snapshotting::<User, String>::json();
    assert_eq!(strat.path_extension.as_deref(), Some("json"));

    let user = User { name: "Alice".into(), age: 30 };
    let result = (strat.snapshot)(&user).await;
    assert!(result.contains("\"name\""));
    assert!(result.contains("\"Alice\""));

    // Keys must be sorted
    let name_pos = result.find("\"age\"").unwrap();
    let age_pos = result.find("\"name\"").unwrap();
    assert!(name_pos < age_pos, "Keys should be sorted alphabetically");
}
```

---

## 6. Diff Algorithm Verification

| Req ID | Requirement | Test File | Test Function | Doc Comment | Status |
|--------|------------|-----------|---------------|-------------|--------|
| REQ-DIFF-ALG-001 | line_diff function | `tests/diff_tests.rs` | `test_line_diff_basic` | YES | DONE |
| REQ-DIFF-ALG-001a | None when identical | `tests/diff_tests.rs` | `test_line_diff_identical` | NEEDED | NOT TESTED |
| REQ-DIFF-ALG-001b | Uses similar crate | N/A (implementation detail) | N/A | YES | DONE |
| REQ-DIFF-ALG-001c | Unified diff format | `tests/diff_tests.rs` | `test_line_diff_format` | NEEDED | NOT TESTED |
| REQ-DIFF-ALG-001d | Configurable context | `tests/diff_tests.rs` | `test_line_diff_context` | NEEDED | NOT TESTED |
| REQ-DIFF-ALG-002 | inline_diff function | `tests/diff_tests.rs` | `test_inline_diff` | YES | DONE |
| REQ-DIFF-ALG-002a | Character-level | `tests/diff_tests.rs` | `test_inline_diff_chars` | NEEDED | NOT TESTED |
| REQ-DIFF-ALG-002b | Inline markers | `tests/diff_tests.rs` | `test_inline_diff_markers` | NEEDED | NOT TESTED |
| REQ-DIFF-ALG-003 | Unicode diff markers | `tests/diff_tests.rs` | `test_diff_unicode_markers` | NEEDED | NOT STARTED |

### Test Plan: Diff

```rust
#[test]
fn test_line_diff_identical() {
    assert!(line_diff("hello\nworld", "hello\nworld", 3).is_none());
}

#[test]
fn test_line_diff_basic() {
    let result = line_diff("hello\nworld", "hello\nearth", 3).unwrap();
    assert!(result.contains("@@"));
    assert!(result.contains("-world"));
    assert!(result.contains("+earth"));
}

#[test]
fn test_line_diff_empty_strings() {
    assert!(line_diff("", "", 3).is_none());
    assert!(line_diff("", "hello", 3).is_some());
    assert!(line_diff("hello", "", 3).is_some());
}

#[test]
fn test_line_diff_context() {
    let old = "a\nb\nc\nd\ne\nf\ng\nh";
    let new = "a\nb\nc\nX\ne\nf\ng\nh";
    let result_1 = line_diff(old, new, 1).unwrap();
    let result_3 = line_diff(old, new, 3).unwrap();
    // More context = more lines in output
    assert!(result_3.len() > result_1.len());
}

#[test]
fn test_inline_diff_basic() {
    let result = inline_diff("hello", "hallo").unwrap();
    assert!(result.contains("Expected:"));
    assert!(result.contains("Actual:"));
}
```

---

## 7. Inline Snapshot Verification

| Req ID | Requirement | Test File | Test Function | Doc Comment | Status |
|--------|------------|-----------|---------------|-------------|--------|
| REQ-INLINE-001 | assert_inline_snapshot macro | `tests/inline_tests.rs` | `test_inline_snapshot_pass` | NEEDED | NOT STARTED |
| REQ-INLINE-001a | Compare with literal | `tests/inline_tests.rs` | `test_inline_compare` | NEEDED | NOT STARTED |
| REQ-INLINE-001b | Record mode rewrites source | `tests/inline_tests.rs` | `test_inline_rewrite` | NEEDED | NOT STARTED |
| REQ-INLINE-001c | Multiline support | `tests/inline_tests.rs` | `test_inline_multiline` | NEEDED | NOT STARTED |
| REQ-INLINE-001d | Raw string delimiters | `tests/inline_tests.rs` | `test_inline_raw_strings` | NEEDED | NOT STARTED |
| REQ-INLINE-002 | Source rewriting | `tests/inline_tests.rs` | `test_source_rewrite` | NEEDED | NOT STARTED |
| REQ-INLINE-002a | Uses syn for parsing | N/A (implementation detail) | N/A | NEEDED | NOT STARTED |
| REQ-INLINE-002b | Batch updates at exit | `tests/inline_tests.rs` | `test_batch_update` | NEEDED | NOT STARTED |
| REQ-INLINE-002c | Multiple snapshots per file | `tests/inline_tests.rs` | `test_multiple_inline_per_file` | NEEDED | NOT STARTED |
| REQ-INLINE-002d | Atomic writes | `tests/inline_tests.rs` | `test_atomic_write` | NEEDED | NOT STARTED |
| REQ-INLINE-003 | SyntaxDescriptor | `tests/inline_tests.rs` | `test_syntax_descriptor` | NEEDED | NOT STARTED |

---

## 8. Custom Dump Verification

| Req ID | Requirement | Test File | Test Function | Doc Comment | Status |
|--------|------------|-----------|---------------|-------------|--------|
| REQ-CDUMP-001 | custom_dump strategy | `tests/custom_dump_tests.rs` | `test_custom_dump_basic` | NEEDED | NOT STARTED |
| REQ-CDUMP-001a | Deterministic output | `tests/custom_dump_tests.rs` | `test_custom_dump_deterministic` | NEEDED | NOT STARTED |
| REQ-CDUMP-001b | Strips pointers | `tests/custom_dump_tests.rs` | `test_custom_dump_no_pointers` | NEEDED | NOT STARTED |
| REQ-CDUMP-001c | Sorted collections | `tests/custom_dump_tests.rs` | `test_custom_dump_sorted` | NEEDED | NOT STARTED |
| REQ-CDUMP-001d | Pretty Debug baseline | `tests/custom_dump_tests.rs` | `test_custom_dump_uses_debug` | NEEDED | NOT STARTED |
| REQ-CDUMP-002 | SnapshotDisplay trait | `tests/custom_dump_tests.rs` | `test_snapshot_display_trait` | NEEDED | NOT STARTED |
| REQ-CDUMP-002a | Custom description | `tests/custom_dump_tests.rs` | `test_snapshot_display_custom` | NEEDED | NOT STARTED |
| REQ-CDUMP-002b | Default render_children | `tests/custom_dump_tests.rs` | `test_snapshot_display_children` | NEEDED | NOT STARTED |

---

## 9. File System Verification

| Req ID | Requirement | Test File | Test Function | Doc Comment | Status |
|--------|------------|-----------|---------------|-------------|--------|
| REQ-FS-001 | Default directory layout | `tests/fs_tests.rs` | `test_default_directory` | NEEDED | NOT TESTED |
| REQ-FS-001a | __snapshots__ next to source | `tests/fs_tests.rs` | `test_snapshots_dir_location` | NEEDED | NOT TESTED |
| REQ-FS-001b | Subdirectory per file | `tests/fs_tests.rs` | `test_subdirectory_per_file` | NEEDED | NOT TESTED |
| REQ-FS-001c | Auto-create directories | `tests/fs_tests.rs` | `test_auto_create_dirs` | NEEDED | DONE |
| REQ-FS-002 | Custom snapshot dir | `tests/fs_tests.rs` | `test_custom_snapshot_dir` | NEEDED | NOT TESTED |
| REQ-FS-003 | Artifact output | `tests/fs_tests.rs` | `test_artifact_output` | NEEDED | NOT STARTED |
| REQ-FS-003a | Writes to artifacts dir | `tests/fs_tests.rs` | `test_artifact_write_location` | NEEDED | NOT STARTED |
| REQ-FS-003b | Uses env var for dir | `tests/fs_tests.rs` | `test_artifact_env_var` | NEEDED | NOT STARTED |

---

## 10. Concurrency Verification

| Req ID | Requirement | Test File | Test Function | Doc Comment | Status |
|--------|------------|-----------|---------------|-------------|--------|
| REQ-THREAD-001 | Thread-local config | `tests/concurrency_tests.rs` | `test_thread_local_config` | YES | DONE |
| REQ-THREAD-001a | Per-thread storage | `tests/concurrency_tests.rs` | `test_per_thread_isolation` | NEEDED | NOT TESTED |
| REQ-THREAD-001b | Push/pop stack | `tests/concurrency_tests.rs` | `test_config_stack` | NEEDED | DONE |
| REQ-THREAD-001c | Drop guard | `tests/concurrency_tests.rs` | `test_drop_guard_on_panic` | NEEDED | NOT STARTED |
| REQ-THREAD-002 | Thread-local counter | `tests/concurrency_tests.rs` | `test_thread_local_counter` | NEEDED | NOT STARTED |
| REQ-THREAD-002a | Counter per thread | `tests/concurrency_tests.rs` | `test_counter_isolation` | NEEDED | NOT STARTED |
| REQ-THREAD-002b | Counter reset | `tests/concurrency_tests.rs` | `test_counter_reset` | NEEDED | NOT STARTED |
| REQ-THREAD-003 | Async execution | `tests/concurrency_tests.rs` | `test_async_verify` | NEEDED | DONE |
| REQ-THREAD-003a | Timeout enforced | `tests/concurrency_tests.rs` | `test_async_timeout` | NEEDED | NOT STARTED |

### Test Plan: Concurrency

```rust
#[test]
fn test_per_thread_isolation() {
    use std::thread;

    with_snapshot_testing(
        SnapshotTestingConfiguration { record: Some(Record::All), ..Default::default() },
        || {
            assert_eq!(current_record(), Record::All);

            let handle = thread::spawn(|| {
                // Different thread should NOT see Record::All
                assert_eq!(current_record(), Record::Missing); // default
            });
            handle.join().unwrap();

            // Original thread still has Record::All
            assert_eq!(current_record(), Record::All);
        },
    );
}
```

---

## 11. Error Handling Verification

| Req ID | Requirement | Test File | Test Function | Doc Comment | Status |
|--------|------------|-----------|---------------|-------------|--------|
| REQ-ERR-001 | SnapshotError enum | `tests/error_tests.rs` | `test_error_variants` | YES | PARTIAL |
| REQ-ERR-002 | Error message format | `tests/error_tests.rs` | `test_error_messages` | NEEDED | NOT TESTED |
| REQ-ERR-002a | Includes snapshot name | `tests/error_tests.rs` | `test_error_includes_name` | NEEDED | NOT STARTED |
| REQ-ERR-002b | Includes diff output | `tests/error_tests.rs` | `test_error_includes_diff` | NEEDED | NOT STARTED |
| REQ-ERR-002c | Includes diff tool cmd | `tests/error_tests.rs` | `test_error_includes_difftool` | NEEDED | NOT STARTED |
| REQ-ERR-002d | Includes file paths | `tests/error_tests.rs` | `test_error_includes_paths` | NEEDED | NOT STARTED |

---

## 12. Environment Variable Verification

| Req ID | Requirement | Test File | Test Function | Doc Comment | Status |
|--------|------------|-----------|---------------|-------------|--------|
| REQ-ENV-001 | SNAPSHOT_TESTING_RECORD | `tests/env_tests.rs` | `test_record_env_var` | YES | DONE |
| REQ-ENV-001 | all variants | `tests/env_tests.rs` | `test_record_env_all_variants` | NEEDED | NOT TESTED |
| REQ-ENV-002 | SNAPSHOT_ARTIFACTS | `tests/env_tests.rs` | `test_artifacts_env_var` | NEEDED | NOT STARTED |

---

## 13. Identified Gaps

### 13.1 Critical Gaps (Must Fix Before v1.0.0)

| Gap | Description | Impact | Priority |
|-----|------------|--------|----------|
| G-001 | No tests exist yet | Cannot verify any behavior | P0 |
| G-002 | assert_snapshot is a function, not a macro | Cannot capture file/line automatically | P0 |
| G-003 | No snapshot counter | Multiple unnamed snapshots per test will overwrite each other | P0 |
| G-004 | No panic-safe config guard | Config leaks if test panics | P0 |
| G-005 | No built-in strategies (lines, json, etc.) | Core use cases not available | P0 |
| G-006 | No timeout support | Async snapshots can hang forever | P1 |
| G-007 | No artifact output on failure | Debugging failures is harder | P1 |
| G-008 | No path sanitization function | Test names with special chars produce invalid paths | P1 |

### 13.2 Medium Gaps

| Gap | Description | Impact | Priority |
|-----|------------|--------|----------|
| G-009 | No inline snapshot support | Must manage snapshot files manually | P2 |
| G-010 | No custom dump strategy | Less readable snapshots | P2 |
| G-011 | No SimplySnapshotting type alias | Minor ergonomic issue | P2 |
| G-012 | No async_pullback | Cannot compose async transforms | P2 |
| G-013 | No Snapshotting::wait() | Cannot wait for async rendering | P2 |
| G-014 | verify_snapshot returns Ok on record (should return Err(Recorded)) | First runs silently pass instead of alerting about new snapshots | P1 |

### 13.3 Low Priority Gaps

| Gap | Description | Impact | Priority |
|-----|------------|--------|----------|
| G-015 | No HTTP request strategy | Must write custom strategy | P3 |
| G-016 | No function strategy (CaseIterable equivalent) | Must write custom strategy | P3 |
| G-017 | No ksdiff DiffTool preset | Minor convenience | P3 |
| G-018 | No DiffTool::from_command string literal support | Minor convenience | P3 |

### 13.4 Doc Comment Coverage Gaps

The following public items need doc comments with examples:

| Item | File | Has Doc? |
|------|------|----------|
| `Snapshotting::new_sync` | snapshotting.rs | N/A (not implemented) |
| `Snapshotting::identity` | snapshotting.rs | N/A (not implemented) |
| `Snapshotting::pullback` | snapshotting.rs | YES |
| `Snapshotting::async_pullback` | snapshotting.rs | N/A (not implemented) |
| `Diffing::new` | diffing.rs | YES |
| `assert_snapshot!` | assert.rs | N/A (not a macro yet) |
| `verify_snapshot` | assert.rs | YES |
| `with_snapshot_testing` | config.rs | YES |
| `current_record` | config.rs | YES |
| `Record::from_env` | config.rs | YES |
| `DiffTool::new` | config.rs | YES |
| `DiffTool::default_tool` | config.rs | YES |
| `line_diff` | diff.rs | YES |
| `inline_diff` | diff.rs | YES |
| All strategy functions | strategies/ | N/A (not implemented) |
