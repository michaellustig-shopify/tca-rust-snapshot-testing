# Software Requirement Specification (SRS)

## rust-snapshot-testing v1.0.0

**Source of truth for what the software must do.**

Swift reference: `swift-snapshot-testing` by Point-Free
Rust workspace: `/Users/michael.lustig/Sync/tca/rust-snapshot-testing/`

---

## Table of Contents

1. [Overview](#1-overview)
2. [Module Structure](#2-module-structure)
3. [Core Types](#3-core-types)
4. [Assertion Functions](#4-assertion-functions)
5. [Configuration System](#5-configuration-system)
6. [Built-in Strategies](#6-built-in-strategies)
7. [Diff Algorithm](#7-diff-algorithm)
8. [Inline Snapshot Testing](#8-inline-snapshot-testing)
9. [Custom Dump](#9-custom-dump)
10. [File System Behavior](#10-file-system-behavior)
11. [Concurrency and Thread Safety](#11-concurrency-and-thread-safety)
12. [Error Handling](#12-error-handling)
13. [Environment Variables](#13-environment-variables)
14. [Platform Considerations](#14-platform-considerations)
15. [Items Explicitly Not Ported](#15-items-explicitly-not-ported)

---

## 1. Overview

### 1.1 Purpose

A Rust port of Point-Free's swift-snapshot-testing library. Provides a composable, strategy-based snapshot testing framework where any type can be snapshot-tested by providing a `Snapshotting<V, F>` strategy.

### 1.2 Design Philosophy

The library uses a struct-based strategy pattern rather than a trait-based approach. This means the same type can have multiple snapshot strategies (e.g., a struct can be snapshot as JSON, as debug text, or as a custom dump). Strategies compose via `pullback` -- a functional operation that transforms one strategy into another.

### 1.3 Scope

Port the **core engine** and **text-based strategies**. Apple platform-specific strategies (UIImage, UIView, NSView, SwiftUI, SceneKit, SpriteKit, WebKit) are out of scope for v1.0.0. They are Apple UI framework APIs with no Rust equivalent.

---

## 2. Module Structure

### 2.1 Crate Layout

| Crate | Swift Module | Purpose |
|-------|-------------|---------|
| `snapshot-testing` | `SnapshotTesting` | Core engine: Snapshotting, Diffing, assert/verify, config |
| `inline-snapshot-testing` | `InlineSnapshotTesting` | Inline snapshot assertions embedded in source code |
| `snapshot-testing-custom-dump` | `SnapshotTestingCustomDump` | Pretty-print strategy using structured debug output |

### 2.2 `snapshot-testing` Internal Modules

| Module | File | Swift Equivalent |
|--------|------|-----------------|
| `snapshotting` | `src/snapshotting.rs` | `Snapshotting.swift` |
| `diffing` | `src/diffing.rs` | `Diffing.swift` |
| `diff` | `src/diff.rs` | `Diff.swift` |
| `assert` | `src/assert.rs` | `AssertSnapshot.swift` |
| `config` | `src/config.rs` | `SnapshotTestingConfiguration.swift` |
| `strategies` | `src/strategies/` | `Snapshotting/*.swift` (NEW -- strategies get a submodule) |

### 2.3 Re-exports at Crate Root

The following must be re-exported from `snapshot_testing::`:

- `Snapshotting<V, F>`
- `Diffing<V>`
- `DiffAttachment`
- `SnapshotTestingConfiguration`
- `Record`
- `DiffTool`
- `assert_snapshot` (macro)
- `verify_snapshot` (function)
- `with_snapshot_testing` (function)
- `line_diff` (function)

---

## 3. Core Types

### 3.1 `Snapshotting<V, F>` [REQ-SNAP-001]

**Swift source**: `Sources/SnapshotTesting/Snapshotting.swift`

A struct that knows how to convert a value of type `V` into a snapshot format `F`, and how to diff and serialize that format.

#### Fields

| Field | Swift Type | Rust Type | Notes |
|-------|-----------|-----------|-------|
| `path_extension` | `String?` | `Option<String>` | File extension for disk storage |
| `diffing` | `Diffing<Format>` | `Diffing<F>` | Comparison/serialization engine |
| `snapshot` | `(Value) -> Async<Format>` | `Arc<dyn Fn(&V) -> Pin<Box<dyn Future<Output = F> + Send>> + Send + Sync>` | Transform function |

#### Constructors

| ID | Swift | Rust | Notes |
|----|-------|------|-------|
| REQ-SNAP-001a | `init(pathExtension:diffing:asyncSnapshot:)` | `Snapshotting::new(path_extension, diffing, snapshot)` where snapshot returns a Future | Async snapshot |
| REQ-SNAP-001b | `init(pathExtension:diffing:snapshot:)` | `Snapshotting::new_sync(path_extension, diffing, snapshot)` | Sync snapshot (wraps in `async { }`) |
| REQ-SNAP-001c | `init(pathExtension:diffing:)` where `Value == Format` | `Snapshotting::identity(path_extension, diffing)` | Identity snapshot -- value IS the format |

#### Methods

| ID | Swift | Rust | Notes |
|----|-------|------|-------|
| REQ-SNAP-002 | `pullback<NewValue>(_ transform: (NewValue) -> Value)` | `fn pullback<NewV>(self, transform: impl Fn(&NewV) -> V) -> Snapshotting<NewV, F>` | Sync pullback |
| REQ-SNAP-003 | `asyncPullback<NewValue>(_ transform: (NewValue) -> Async<Value>)` | `fn async_pullback<NewV>(self, transform: impl Fn(&NewV) -> Pin<Box<dyn Future<Output = V> + Send>>) -> Snapshotting<NewV, F>` | Async pullback |

#### Behavior

- REQ-SNAP-002-B: `pullback` must preserve `path_extension` and `diffing` from the source strategy.
- REQ-SNAP-003-B: `async_pullback` must chain the transform future with the snapshot future -- first transform the value, then snapshot it.
- REQ-SNAP-004: `Snapshotting` must implement `Clone` (via `Arc` on the snapshot closure).

#### Type Alias

| ID | Swift | Rust | Notes |
|----|-------|------|-------|
| REQ-SNAP-005 | `typealias SimplySnapshotting<Format> = Snapshotting<Format, Format>` | `type SimplySnapshotting<F> = Snapshotting<F, F>;` | When value type == format type |

### 3.2 `Diffing<V>` [REQ-DIFF-001]

**Swift source**: `Sources/SnapshotTesting/Diffing.swift`

A struct that knows how to serialize, deserialize, and compare values of type `V`.

#### Fields

| Field | Swift Type | Rust Type | Notes |
|-------|-----------|-----------|-------|
| `to_data` | `(Value) -> Data` | `Arc<dyn Fn(&V) -> Vec<u8> + Send + Sync>` | Serialize to bytes |
| `from_data` | `(Data) -> Value` | `Arc<dyn Fn(&[u8]) -> V + Send + Sync>` | Deserialize from bytes |
| `diff` | `(Value, Value) -> (String, [DiffAttachment])?` | `Arc<dyn Fn(&V, &V) -> Option<(String, Vec<DiffAttachment>)> + Send + Sync>` | Compare two values |

#### Constructors

| ID | Rust | Notes |
|----|------|-------|
| REQ-DIFF-001a | `Diffing::new(to_data, from_data, diff)` | Primary constructor |

#### Behavior

- REQ-DIFF-002: `diff` returns `None` when values are equal, `Some((message, attachments))` when different.
- REQ-DIFF-003: `Diffing` must implement `Clone` (via `Arc`).
- REQ-DIFF-004: `to_data` and `from_data` must be inverse operations: `from_data(to_data(v))` should produce a value equal to `v` (per the diff function).

### 3.3 `DiffAttachment` [REQ-ATTACH-001]

**Swift source**: `Sources/SnapshotTesting/Diffing.swift`

#### Variants

| Swift | Rust | Notes |
|-------|------|-------|
| `.data(Data, name: String)` | `DiffAttachment::Data { bytes: Vec<u8>, name: String }` | Named binary blob |
| `.xcTest(XCTAttachment)` | NOT PORTED | XCTest-specific, deprecated in Swift |

### 3.4 `Async<Value>` [REQ-ASYNC-001]

**Swift source**: `Sources/SnapshotTesting/Async.swift`

In Swift, `Async<Value>` is a callback-based wrapper. In Rust, we use native `Future` instead.

| Swift | Rust Equivalent | Notes |
|-------|----------------|-------|
| `Async<Value>` | `Pin<Box<dyn Future<Output = V> + Send>>` | Standard async |
| `Async(value: v)` | `async { v }` or `std::future::ready(v)` | Immediate value |
| `Async { callback in ... }` | `async { ... }` | Deferred computation |
| `async.map(transform)` | `.map(transform)` via `FutureExt` | Transform output |
| `async.run { callback($0) }` | `.await` | Execute |

---

## 4. Assertion Functions

### 4.1 `assert_snapshot` [REQ-ASSERT-001]

**Swift source**: `Sources/SnapshotTesting/AssertSnapshot.swift`

Primary user-facing assertion. Panics (in Rust) on mismatch.

#### Swift Signature

```swift
func assertSnapshot<Value, Format>(
    of value: @autoclosure () throws -> Value,
    as snapshotting: Snapshotting<Value, Format>,
    named name: String? = nil,
    record: Record? = nil,
    timeout: TimeInterval = 5,
    fileID: StaticString = #fileID,
    file filePath: StaticString = #filePath,
    testName: String = #function,
    line: UInt = #line,
    column: UInt = #column
)
```

#### Rust API (macro)

```rust
assert_snapshot!(value, as: snapshotting)
assert_snapshot!(value, as: snapshotting, named: "variant")
assert_snapshot!(value, as: snapshotting, record: Record::All)
```

The macro must:
- REQ-ASSERT-001a: Capture `file!()`, `line!()`, `module_path!()` automatically.
- REQ-ASSERT-001b: Derive the snapshot directory from the source file location: `<test_file_dir>/__snapshots__/<test_file_name>/`
- REQ-ASSERT-001c: Derive the test name from `module_path!()` and the function name.
- REQ-ASSERT-001d: Call `verify_snapshot` internally and panic with the error message on failure.
- REQ-ASSERT-001e: Support an optional `timeout` parameter (default 5 seconds).

### 4.2 `verify_snapshot` [REQ-VERIFY-001]

**Swift source**: `Sources/SnapshotTesting/AssertSnapshot.swift`

The testable core. Returns a `Result` instead of panicking.

#### Rust Signature

```rust
pub async fn verify_snapshot<V, F>(
    value: &V,
    snapshotting: &Snapshotting<V, F>,
    name: Option<&str>,
    record: Option<Record>,
    snapshot_dir: &Path,
    test_name: &str,
    timeout: Duration,
) -> Result<(), SnapshotError>
where
    F: Clone + Send + 'static,
```

#### Behavior Specification

| Step | Condition | Action | Requirement |
|------|-----------|--------|-------------|
| 1 | Always | Generate snapshot via `snapshotting.snapshot(value)` with timeout | REQ-VERIFY-001a |
| 2 | Timeout exceeded | Return `Err(SnapshotError::Timeout)` | REQ-VERIFY-001b |
| 3 | Snapshot generation fails | Return `Err(SnapshotError::SnapshotFailed)` | REQ-VERIFY-001c |
| 4 | `record == All` | Write snapshot to disk, return message about recording | REQ-VERIFY-001d |
| 5 | Reference file missing, `record == Never` | Return `Err(MissingSnapshot)`, do NOT write | REQ-VERIFY-001e |
| 6 | Reference file missing, `record != Never` | Write snapshot to disk, return message about recording | REQ-VERIFY-001f |
| 7 | Reference exists, diff matches | Return `Ok(())` | REQ-VERIFY-001g |
| 8 | Reference exists, diff mismatches, `record == Failed` | Write new snapshot, return mismatch message | REQ-VERIFY-001h |
| 9 | Reference exists, diff mismatches, other modes | Return `Err(Mismatch)` with diff | REQ-VERIFY-001i |

#### Snapshot File Naming

| Component | Rule | Requirement |
|-----------|------|-------------|
| Directory | `<source_dir>/__snapshots__/<source_file_stem>/` | REQ-VERIFY-002a |
| File name (named) | `<test_name>.<sanitized_name>.<ext>` | REQ-VERIFY-002b |
| File name (unnamed, first) | `<test_name>.1.<ext>` | REQ-VERIFY-002c |
| File name (unnamed, nth) | `<test_name>.<n>.<ext>` (auto-incrementing counter) | REQ-VERIFY-002d |
| Path sanitization | Replace `\W+` with `-`, strip leading/trailing `-` | REQ-VERIFY-002e |

#### Snapshot Counter [REQ-COUNTER-001]

- REQ-COUNTER-001a: Maintain a thread-safe counter mapping `snapshot_dir + test_name` to an incrementing integer.
- REQ-COUNTER-001b: Counter resets between test cases (not between assertions within the same test).
- REQ-COUNTER-001c: Counter is per-thread (thread-local) to avoid races in parallel test execution.

### 4.3 `assert_snapshots` (multiple strategies) [REQ-ASSERT-002]

| Swift | Rust | Notes |
|-------|------|-------|
| `assertSnapshots(of:as: [String: Snapshotting])` | `assert_snapshots!(value, as: { "name1" => strategy1, "name2" => strategy2 })` | Named strategies |
| `assertSnapshots(of:as: [Snapshotting])` | `assert_snapshots!(value, as: [strategy1, strategy2])` | Unnamed strategies |

### 4.4 Failure Artifact Output [REQ-ARTIFACT-001]

When a diff fails:

- REQ-ARTIFACT-001a: Write the actual (failed) snapshot to the artifacts directory.
- REQ-ARTIFACT-001b: Artifacts directory is `SNAPSHOT_ARTIFACTS` env var, or `std::env::temp_dir()` if unset.
- REQ-ARTIFACT-001c: Include the diff tool command in the failure message.

---

## 5. Configuration System

### 5.1 `SnapshotTestingConfiguration` [REQ-CONFIG-001]

| Field | Type | Default | Notes |
|-------|------|---------|-------|
| `record` | `Option<Record>` | `None` | Recording mode |
| `diff_tool` | `Option<DiffTool>` | `None` | Diff tool for failure messages |

### 5.2 `Record` [REQ-RECORD-001]

| Variant | Behavior |
|---------|----------|
| `All` | Always write snapshots to disk |
| `Failed` | Write only when comparison fails |
| `Missing` | Write only when file does not exist (DEFAULT) |
| `Never` | Never write; fail if missing (CI mode) |

- REQ-RECORD-001a: Parse from `SNAPSHOT_TESTING_RECORD` env var.
- REQ-RECORD-001b: Invalid env var value must produce a clear panic.
- REQ-RECORD-001c: `Record` must implement `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`.

### 5.3 `DiffTool` [REQ-DIFFTOOL-001]

A callable that takes `(current_file_path, failed_file_path)` and returns a shell command string.

| ID | Preset | Output |
|----|--------|--------|
| REQ-DIFFTOOL-001a | `DiffTool::default_tool()` | Prints `file://` URLs |
| REQ-DIFFTOOL-001b | `DiffTool::ksdiff()` | `ksdiff "<current>" "<failed>"` |
| REQ-DIFFTOOL-001c | `DiffTool::new(closure)` | Custom closure |

- REQ-DIFFTOOL-002: `DiffTool` must implement `Clone` (via `Arc`).

### 5.4 `with_snapshot_testing` [REQ-SCOPE-001]

**Swift source**: `Sources/SnapshotTesting/SnapshotTestingConfiguration.swift`

```rust
pub fn with_snapshot_testing<R>(
    config: SnapshotTestingConfiguration,
    f: impl FnOnce() -> R,
) -> R
```

- REQ-SCOPE-001a: Configuration is scoped to the closure.
- REQ-SCOPE-001b: Calls can be nested; inner config overrides outer.
- REQ-SCOPE-001c: Uses thread-local storage (not global state).
- REQ-SCOPE-001d: Configuration is restored even if the closure panics (use RAII guard).

### 5.5 Configuration Resolution Priority [REQ-CONFIG-002]

1. Explicit `record` parameter on `verify_snapshot` / `assert_snapshot`
2. Thread-local config from `with_snapshot_testing` (innermost wins)
3. `SNAPSHOT_TESTING_RECORD` environment variable
4. Default: `Record::Missing`

---

## 6. Built-in Strategies

### 6.1 `Snapshotting::lines()` [REQ-STRAT-001]

**Swift**: `Snapshotting<String, String>.lines`

```rust
impl Snapshotting<String, String> {
    pub fn lines() -> Self { ... }
}
```

- REQ-STRAT-001a: `path_extension` = `"txt"`
- REQ-STRAT-001b: `to_data` = UTF-8 encode
- REQ-STRAT-001c: `from_data` = UTF-8 decode
- REQ-STRAT-001d: `diff` = line-level unified diff (see section 7)

### 6.2 `Diffing::lines()` [REQ-STRAT-002]

**Swift**: `Diffing<String>.lines`

```rust
impl Diffing<String> {
    pub fn lines() -> Self { ... }
}
```

- REQ-STRAT-002a: Returns `None` when strings are identical.
- REQ-STRAT-002b: Produces unified diff hunks with `@@` markers.
- REQ-STRAT-002c: Includes a `DiffAttachment::Data` named `"difference.patch"` containing the diff.

### 6.3 `Snapshotting::description()` [REQ-STRAT-003]

**Swift**: `Snapshotting<Value, String>.description` (where `Format == String`)

```rust
impl<V: std::fmt::Display> Snapshotting<V, String> {
    pub fn description() -> Self { ... }
}
```

- REQ-STRAT-003a: Uses `Display` trait (Swift's `String(describing:)`).
- REQ-STRAT-003b: Pulls back `Snapshotting::lines()` through `Display::to_string`.

### 6.4 `Snapshotting::debug()` [REQ-STRAT-004]

**Swift**: `Snapshotting<Value, String>.dump` (uses `Mirror`-based dump)

```rust
impl<V: std::fmt::Debug> Snapshotting<V, String> {
    pub fn debug() -> Self { ... }
}
```

- REQ-STRAT-004a: Uses `Debug` trait (closest Rust equivalent to Swift's `dump`).
- REQ-STRAT-004b: Pulls back `Snapshotting::lines()` through `format!("{:#?}", value)` (pretty-printed debug).

### 6.5 `Snapshotting::json()` [REQ-STRAT-005]

**Swift**: `Snapshotting<Value, String>.json` where `Value: Encodable`

```rust
impl<V: serde::Serialize> Snapshotting<V, String> {
    pub fn json() -> Self { ... }
    pub fn json_with(settings: JsonSettings) -> Self { ... }
}
```

- REQ-STRAT-005a: `path_extension` = `"json"`
- REQ-STRAT-005b: Default: pretty-printed with sorted keys.
- REQ-STRAT-005c: `json_with` allows custom `serde_json::ser::PrettyFormatter` settings.

### 6.6 `Snapshotting::data()` [REQ-STRAT-006]

**Swift**: `Snapshotting<Data, Data>.data`

```rust
impl Snapshotting<Vec<u8>, Vec<u8>> {
    pub fn data() -> Self { ... }
}
```

- REQ-STRAT-006a: `path_extension` = `None`
- REQ-STRAT-006b: `to_data` / `from_data` = identity
- REQ-STRAT-006c: `diff` = byte equality check; message includes sizes if different.

### 6.7 `Snapshotting::func()` (CaseIterable) [REQ-STRAT-007]

**Swift**: `Snapshotting.func(into:)` where `Value: CaseIterable`

```rust
// For enums that implement strum::IntoEnumIterator or similar
impl<V, A> Snapshotting<Box<dyn Fn(V) -> A>, String>
where
    V: IntoIterator + Clone,
{
    pub fn function(witness: Snapshotting<A, String>) -> Self { ... }
}
```

- REQ-STRAT-007a: Feeds every input variant into the function and records input/output pairs as CSV.
- REQ-STRAT-007b: `path_extension` = `"csv"`

### 6.8 `Snapshotting::raw()` (HTTP Request) [REQ-STRAT-008]

**Swift**: `Snapshotting<URLRequest, String>.raw`

```rust
// Using the `http` crate's Request type
impl<B: AsRef<[u8]>> Snapshotting<http::Request<B>, String> {
    pub fn raw() -> Self { ... }
    pub fn raw_pretty() -> Self { ... }
    pub fn curl() -> Self { ... }
}
```

- REQ-STRAT-008a: `.raw()` = Method + URL + headers + body as text.
- REQ-STRAT-008b: `.raw_pretty()` = Same but attempts JSON pretty-print on body.
- REQ-STRAT-008c: `.curl()` = Outputs a `curl` command that reproduces the request.
- REQ-STRAT-008d: Query parameters sorted alphabetically for determinism.
- REQ-STRAT-008e: Headers sorted alphabetically for determinism.

### 6.9 `Snapshotting::wait()` [REQ-STRAT-009]

**Swift**: `Snapshotting.wait(for:on:)`

```rust
impl<V, F> Snapshotting<V, F>
where
    F: Clone + Send + 'static,
    V: 'static,
{
    pub fn wait(duration: Duration, strategy: Self) -> Self { ... }
}
```

- REQ-STRAT-009a: Wraps another strategy and waits `duration` before taking the snapshot.
- REQ-STRAT-009b: Useful for async rendering scenarios.

---

## 7. Diff Algorithm

### 7.1 `line_diff` [REQ-DIFF-ALG-001]

**Swift source**: `Sources/SnapshotTesting/Diff.swift`

```rust
pub fn line_diff(old: &str, new: &str, context_lines: usize) -> Option<String>
```

- REQ-DIFF-ALG-001a: Returns `None` if strings are identical.
- REQ-DIFF-ALG-001b: Uses the `similar` crate (Myers diff algorithm) instead of the hand-rolled LCS in Swift.
- REQ-DIFF-ALG-001c: Produces unified diff format with `@@` hunk markers.
- REQ-DIFF-ALG-001d: Default context of 3 lines (configurable).

### 7.2 `inline_diff` [REQ-DIFF-ALG-002]

```rust
pub fn inline_diff(old: &str, new: &str) -> Option<String>
```

- REQ-DIFF-ALG-002a: Character-level diff for short strings.
- REQ-DIFF-ALG-002b: Shows `[deleted]` and `[inserted]` inline markers.

### 7.3 Special Characters in Diff Output [REQ-DIFF-ALG-003]

The Swift library uses special Unicode characters in diff output:

| Character | Unicode | Swift Constant | Rust Constant | Purpose |
|-----------|---------|---------------|---------------|---------|
| Minus sign | U+2212 | `minus` | `MINUS` | Deletion marker (not ASCII hyphen) |
| Plus sign | U+002B | `plus` | `PLUS` | Addition marker |
| Figure space | U+2007 | `figureSpace` | `FIGURE_SPACE` | Context line prefix (same width as +/-) |

- REQ-DIFF-ALG-003a: Diff output must use these characters for visual alignment.

---

## 8. Inline Snapshot Testing

### 8.1 `assert_inline_snapshot!` [REQ-INLINE-001]

**Swift source**: `Sources/InlineSnapshotTesting/AssertInlineSnapshot.swift`

```rust
assert_inline_snapshot!(
    value,
    as: snapshotting,
    @"expected output"  // literal string
);
```

- REQ-INLINE-001a: Compare the snapshot against the string literal in the source code.
- REQ-INLINE-001b: In record mode, rewrite the source file to insert/update the expected string.
- REQ-INLINE-001c: Support multiline string literals.
- REQ-INLINE-001d: Must handle the `#` raw string delimiters correctly when the snapshot contains `"`.

### 8.2 Source Rewriting [REQ-INLINE-002]

- REQ-INLINE-002a: Use `syn` and `proc-macro2` (Rust equivalents of Swift's `swift-syntax`) to parse and rewrite test source files.
- REQ-INLINE-002b: Rewriting happens at process exit (atexit hook), batching all updates per file.
- REQ-INLINE-002c: Multiple inline snapshots in the same test file must all be updated correctly (track line offsets).
- REQ-INLINE-002d: Must not corrupt the source file -- use atomic writes.

### 8.3 `InlineSnapshotSyntaxDescriptor` [REQ-INLINE-003]

```rust
pub struct InlineSnapshotSyntaxDescriptor {
    pub trailing_closure_label: String,
    pub trailing_closure_offset: usize,
    pub deprecated_trailing_closure_labels: Vec<String>,
}
```

- REQ-INLINE-003a: Allows custom snapshot helper functions to specify where their inline snapshot lives in the syntax tree.
- REQ-INLINE-003b: Default `trailing_closure_label` = `"matches"`.

---

## 9. Custom Dump

### 9.1 `Snapshotting::custom_dump()` [REQ-CDUMP-001]

**Swift source**: `Sources/SnapshotTestingCustomDump/CustomDump.swift`

```rust
impl<V: std::fmt::Debug> Snapshotting<V, String> {
    pub fn custom_dump() -> Self { ... }
}
```

- REQ-CDUMP-001a: Produces deterministic, human-readable output.
- REQ-CDUMP-001b: Strips pointer addresses from output.
- REQ-CDUMP-001c: Sorts dictionary/set contents for determinism.
- REQ-CDUMP-001d: Uses pretty-printed `Debug` output (`{:#?}`) as the baseline.
- REQ-CDUMP-001e: Pulls back through `Snapshotting::lines()`.

### 9.2 `AnySnapshotStringConvertible` equivalent [REQ-CDUMP-002]

**Swift**: protocol `AnySnapshotStringConvertible`

In Rust, this becomes an optional trait:

```rust
pub trait SnapshotDisplay {
    fn snapshot_description(&self) -> String;
    fn render_children() -> bool { false }
}
```

- REQ-CDUMP-002a: Types implementing `SnapshotDisplay` use `snapshot_description()` in custom dump.
- REQ-CDUMP-002b: Default `render_children()` returns `false` (leaf node).

---

## 10. File System Behavior

### 10.1 Snapshot Directory Layout [REQ-FS-001]

```
tests/
  my_tests.rs
  __snapshots__/
    my_tests/
      test_name.1.txt
      test_name.some_variant.json
```

- REQ-FS-001a: `__snapshots__` directory lives next to the test source file.
- REQ-FS-001b: Subdirectory named after the test file (without extension).
- REQ-FS-001c: Directories are created automatically when recording.

### 10.2 Custom Snapshot Directory [REQ-FS-002]

- REQ-FS-002a: `verify_snapshot` accepts an optional `snapshot_dir` override.
- REQ-FS-002b: When provided, it replaces the default `__snapshots__/<file>/` path entirely.

### 10.3 Artifact Output [REQ-FS-003]

- REQ-FS-003a: Failed snapshots are written to the artifacts directory.
- REQ-FS-003b: Artifacts directory = `SNAPSHOT_ARTIFACTS` env var, or `std::env::temp_dir()`.
- REQ-FS-003c: Artifact subdirectory = test file name (without extension).

---

## 11. Concurrency and Thread Safety

### 11.1 Thread-Local Configuration [REQ-THREAD-001]

- REQ-THREAD-001a: `SnapshotTestingConfiguration` stored in thread-local storage (not global).
- REQ-THREAD-001b: `with_snapshot_testing` pushes/pops a config stack per thread.
- REQ-THREAD-001c: Stack unwinding must restore config even on panic (use a drop guard).

### 11.2 Snapshot Counter [REQ-THREAD-002]

- REQ-THREAD-002a: Counter is thread-local to avoid races in parallel test execution.
- REQ-THREAD-002b: Counter resets between test functions.

### 11.3 Async Snapshot Execution [REQ-THREAD-003]

- REQ-THREAD-003a: `verify_snapshot` is an async function that awaits the snapshot future.
- REQ-THREAD-003b: Timeout enforced via `tokio::time::timeout` or `futures::future::timeout`.
- REQ-THREAD-003c: Must work with both `tokio` and `async-std` runtimes (use `futures` crate for portability, or provide feature flags).

---

## 12. Error Handling

### 12.1 `SnapshotError` [REQ-ERR-001]

```rust
#[derive(Debug, thiserror::Error)]
pub enum SnapshotError {
    #[error("Snapshot mismatch:\n{message}\n\n{diff_tool_output}")]
    Mismatch {
        message: String,
        diff_tool_output: String,
    },

    #[error("No reference snapshot found at {path}")]
    MissingSnapshot { path: PathBuf },

    #[error("Snapshot recorded: {path}")]
    Recorded { path: PathBuf },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Snapshot timed out after {seconds}s")]
    Timeout { seconds: f64 },

    #[error("Snapshot generation failed")]
    SnapshotFailed,
}
```

### 12.2 Error Message Format [REQ-ERR-002]

Mismatch messages must include:

- REQ-ERR-002a: The snapshot name (if provided).
- REQ-ERR-002b: The diff output.
- REQ-ERR-002c: The diff tool command (from `DiffTool`).
- REQ-ERR-002d: Paths to expected and actual files.

---

## 13. Environment Variables

| Variable | Values | Default | Effect | Requirement |
|----------|--------|---------|--------|-------------|
| `SNAPSHOT_TESTING_RECORD` | `all`, `missing`, `failed`, `never` | `missing` | Sets global record mode | REQ-ENV-001 |
| `SNAPSHOT_ARTIFACTS` | Directory path | `std::env::temp_dir()` | Where failed snapshot artifacts are written | REQ-ENV-002 |

---

## 14. Platform Considerations

### 14.1 Cross-Platform Support [REQ-PLATFORM-001]

- REQ-PLATFORM-001a: Core library must compile on Linux, macOS, and Windows.
- REQ-PLATFORM-001b: Use `cfg(target_os)` where needed (file path separators, temp directories).
- REQ-PLATFORM-001c: No Apple framework dependencies in the core crate.

### 14.2 Conditional Compilation Mapping [REQ-PLATFORM-002]

| Swift | Rust |
|-------|------|
| `#if os(macOS)` | `#[cfg(target_os = "macos")]` |
| `#if os(iOS)` | N/A (no iOS Rust target in scope) |
| `#if os(Linux)` | `#[cfg(target_os = "linux")]` |
| `#if os(Windows)` | `#[cfg(target_os = "windows")]` |
| `#if canImport(UIKit)` | N/A |

---

## 15. Items Explicitly Not Ported

These are Apple platform-specific and have no Rust equivalent:

| Swift File | Reason |
|-----------|--------|
| `CALayer.swift` | Core Animation -- Apple only |
| `CGPath.swift` | Core Graphics -- Apple only |
| `NSBezierPath.swift` | AppKit -- Apple only |
| `NSImage.swift` | AppKit -- Apple only |
| `NSView.swift` | AppKit -- Apple only |
| `NSViewController.swift` | AppKit -- Apple only |
| `UIBezierPath.swift` | UIKit -- Apple only |
| `UIImage.swift` | UIKit -- Apple only |
| `UIView.swift` | UIKit -- Apple only |
| `UIViewController.swift` | UIKit -- Apple only |
| `SwiftUIView.swift` | SwiftUI -- Apple only |
| `SceneKit.swift` | SceneKit -- Apple only |
| `SpriteKit.swift` | SpriteKit -- Apple only |
| `View.swift` (ViewImageConfig) | UIKit/AppKit device configs -- Apple only |
| `PlistEncoder.swift` | Property list format -- not useful outside Apple |
| `XCTAttachment.swift` | XCTest-specific |
| `RecordIssue.swift` | XCTest/Swift Testing-specific issue reporting |
| `SnapshotsTestTrait.swift` | Swift Testing `Trait` protocol -- no Rust equivalent |
| `Deprecations.swift` | Swift API migration helpers -- not applicable |

The **Plist strategy** (`Snapshotting.plist`) is not ported because Property Lists are an Apple-specific format. If needed later, it can be added behind a feature flag using the `plist` crate.

---

## Current Implementation Status

| Requirement | Status | Notes |
|-------------|--------|-------|
| REQ-SNAP-001 (Snapshotting struct) | PARTIAL | Struct exists, missing `new_sync` and `identity` constructors |
| REQ-SNAP-002 (pullback) | PARTIAL | Exists but takes `Fn(&NewV) -> V`, needs ownership review |
| REQ-SNAP-003 (async_pullback) | NOT STARTED | |
| REQ-SNAP-005 (SimplySnapshotting) | NOT STARTED | |
| REQ-DIFF-001 (Diffing struct) | DONE | |
| REQ-ATTACH-001 (DiffAttachment) | DONE | |
| REQ-ASSERT-001 (assert_snapshot macro) | PARTIAL | Function exists, not a macro yet |
| REQ-VERIFY-001 (verify_snapshot) | PARTIAL | Core logic exists, missing timeout, counter, artifact output |
| REQ-COUNTER-001 (snapshot counter) | NOT STARTED | |
| REQ-CONFIG-001 (Configuration) | DONE | |
| REQ-RECORD-001 (Record enum) | DONE | |
| REQ-DIFFTOOL-001 (DiffTool) | PARTIAL | Missing ksdiff preset |
| REQ-SCOPE-001 (with_snapshot_testing) | DONE | Missing panic-safe drop guard |
| REQ-STRAT-001 (lines) | NOT STARTED | |
| REQ-STRAT-002 (Diffing::lines) | NOT STARTED | |
| REQ-STRAT-003 (description) | NOT STARTED | |
| REQ-STRAT-004 (debug) | NOT STARTED | |
| REQ-STRAT-005 (json) | NOT STARTED | |
| REQ-STRAT-006 (data) | NOT STARTED | |
| REQ-STRAT-007 (func/CaseIterable) | NOT STARTED | |
| REQ-STRAT-008 (HTTP request) | NOT STARTED | |
| REQ-STRAT-009 (wait) | NOT STARTED | |
| REQ-DIFF-ALG-001 (line_diff) | DONE | |
| REQ-DIFF-ALG-002 (inline_diff) | DONE | |
| REQ-INLINE-001 (assert_inline_snapshot) | NOT STARTED | |
| REQ-INLINE-002 (source rewriting) | NOT STARTED | |
| REQ-CDUMP-001 (custom_dump strategy) | NOT STARTED | |
| REQ-ERR-001 (SnapshotError) | PARTIAL | Missing Recorded variant and diff tool output |
| REQ-ENV-001 (SNAPSHOT_TESTING_RECORD) | DONE | |
| REQ-ENV-002 (SNAPSHOT_ARTIFACTS) | NOT STARTED | |
