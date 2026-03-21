# Lesson 08: Testing in Rust

## The Big Idea

Rust has a built-in test framework -- no separate test target or XCTest import needed. You annotate functions with `#[test]` and run `cargo test`. But Rust also has some unique features: doc tests (code examples in comments that actually compile and run), integration tests in a separate `tests/` directory, and the `#[ignore]` attribute for skipping tests.

This project uses all of these patterns. Let's see how they map to what you know from Swift/XCTest.

## `#[test]`: The Basic Test

```rust
// From crates/snapshot-testing/tests/deprecation_tests.rs
#[test]
fn test_is_recording_proxy() {
    let config_all = SnapshotTestingConfiguration {
        record: Some(Record::All),
        diff_tool: None,
    };
    with_snapshot_testing(config_all, || {
        assert_eq!(current_record(), Record::All);
    });
}
```

**Swift equivalent:**
```swift
func testIsRecordingProxy() {
    // XCTest: method name starts with `test`
    let config = SnapshotTestingConfiguration(record: .all)
    withSnapshotTesting(config) {
        XCTAssertEqual(currentRecord(), .all)
    }
}
```

Key differences:
- Rust: `#[test]` attribute, any function name, `assert_eq!` macro
- Swift: method name starts with `test`, `XCTAssertEqual` function

## `#[ignore]`: Skipping Tests

This project has many tests marked `#[ignore]` because they're ported from Swift but the implementations aren't complete yet:

```rust
// From crates/snapshot-testing/tests/snapshot_testing_tests.rs
#[test]
#[ignore] // TODO: implement Snapshotting::dump()
fn test_any() {
    #[derive(Debug)]
    struct User {
        id: i32,
        name: String,
        bio: String,
    }
    // ...
}
```

Run ignored tests explicitly:
```bash
cargo test -- --ignored           # run ONLY ignored tests
cargo test -- --include-ignored   # run ALL tests including ignored
```

**Swift equivalent:** There's no direct `XCTest` equivalent. You'd either comment out the test or use `XCTSkip`:
```swift
func testAny() throws {
    throw XCTSkip("TODO: implement dump strategy")
}
```

## `#[cfg(test)]`: Conditional Compilation

```rust
// From crates/snapshot-testing/tests/snapshot_testing_tests.rs
#[cfg(test)]
mod snapshot_testing_tests {
    use snapshot_testing::*;
    // ... tests
}
```

`#[cfg(test)]` means "only compile this code when running tests." This is useful for test-only modules inside your source files. For files in the `tests/` directory, it's technically not needed (they're always test code), but it's a common convention.

**Swift equivalent:** In Swift, test targets are separate. There's no in-source conditional compilation for tests.

## Test Organization: Unit Tests vs Integration Tests

Rust has two places for tests:

### Unit Tests (inside `src/`)

```rust
// In src/config.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_from_env() {
        // Can access private items from the parent module
    }
}
```

Unit tests live inside the source file, in a `mod tests` block. They can access private items.

### Integration Tests (in `tests/`)

```
crates/snapshot-testing/tests/
    helpers.rs                        -- shared test utilities
    snapshot_testing_tests.rs         -- main test suite
    with_snapshot_testing_tests.rs    -- configuration tests
    record_tests.rs                   -- record mode tests
    ...
```

Integration tests live in the `tests/` directory. Each `.rs` file is compiled as a separate crate that depends on your library. They can only access public items -- they test the public API.

**Swift equivalent:**
```
Sources/SnapshotTesting/           -- source code
Tests/SnapshotTestingTests/        -- test code (separate target)
```

## Test Helpers

The project has a `helpers.rs` file with shared test utilities:

```rust
// From crates/snapshot-testing/tests/helpers.rs
pub fn with_base_config<R>(f: impl FnOnce() -> R) -> R {
    let config = SnapshotTestingConfiguration {
        record: Some(Record::Failed),
        diff_tool: Some(DiffTool::new(|a, b| format!("ksdiff {} {}", a, b))),
    };
    with_snapshot_testing(config, f)
}
```

Other test files use it with `mod helpers;`:

```rust
// From crates/snapshot-testing/tests/snapshot_testing_tests.rs
mod helpers;
```

**Swift equivalent:**
```swift
// BaseTestCase.swift
class BaseTestCase: XCTestCase {
    override func invokeTest() {
        withSnapshotTesting(record: .failed, diffTool: .ksdiff) {
            super.invokeTest()
        }
    }
}
```

In Swift, you'd use a base test class. In Rust, you use a helper function (Rust doesn't have test class inheritance).

## Doc Tests: Code Examples That Run

Look at the `DiffTool` implementation in `config.rs`:

```rust
/// Create a custom diff tool.
///
/// # Examples
///
/// ```
/// use snapshot_testing::config::DiffTool;
/// let tool = DiffTool::new(|a, b| format!("diff {a} {b}"));
/// assert!(tool.command("foo.txt", "bar.txt").contains("diff"));
/// ```
pub fn new<F>(command: F) -> Self { ... }
```

The code block inside `/// ``` ... ``` ` is a **doc test**. When you run `cargo test`, Rust compiles and runs every code example in your documentation. If the code doesn't compile or the assertions fail, the test fails.

This is incredibly powerful for documentation quality: your examples can never go stale because they're verified on every test run.

**Swift has no equivalent.** Swift doc comments with code examples are just text -- they're never executed.

Run doc tests specifically:
```bash
cargo test --doc
```

## Platform-Specific Test Constants

The helpers file uses compile-time constants for platform detection:

```rust
// From crates/snapshot-testing/tests/helpers.rs
pub const PLATFORM: &str = if cfg!(target_os = "macos") {
    "macos"
} else if cfg!(target_os = "linux") {
    "linux"
} else {
    "unknown"
};
```

`cfg!()` is a compile-time condition macro. It evaluates at build time, not runtime. This is similar to Swift's `#if os(...)` compiler directives.

## The TempSnapshotDir Pattern

The record tests use a cleanup pattern with `Drop`:

```rust
// From crates/snapshot-testing/tests/record_tests.rs
struct TempSnapshotDir {
    path: PathBuf,
}

impl TempSnapshotDir {
    fn new(test_name: &str) -> Self {
        let path = std::env::temp_dir()
            .join("rust_snapshot_tests")
            .join("__Snapshots__")
            .join("RecordTests");
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create temp snapshot dir");
        TempSnapshotDir { path }
    }
}

impl Drop for TempSnapshotDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
```

When `TempSnapshotDir` goes out of scope, `Drop` runs and cleans up the directory. This is like Swift's `deinit` but for any struct. It's also Rust's version of the `defer` + `addTeardownBlock` pattern in XCTest.

## Running Tests

```bash
# Run all tests
cargo test

# Run tests in a specific crate
cargo test -p snapshot-testing

# Run tests matching a name pattern
cargo test test_record

# Run a specific test
cargo test -- test_record_never

# Run with output (println! visible)
cargo test -- --nocapture

# Run tests in parallel (default) or sequentially
cargo test -- --test-threads=1
```

**Swift equivalent:**
```bash
swift test                        # Run all tests
swift test --filter RecordTests   # Filter by suite name
```

## The `insta` Crate

This project lists `insta` as a dev-dependency:

```toml
# From crates/snapshot-testing/Cargo.toml
[dev-dependencies]
insta = { workspace = true }
```

`insta` is an existing Rust snapshot testing library. We include it as a dev-dependency for testing our own library (you can test a snapshot testing library with another snapshot testing library). It provides `assert_snapshot!` and `assert_yaml_snapshot!` macros with a `cargo insta review` workflow.

## Exercise

Run the exercise for this lesson:

```bash
cargo run -p snapshot-testing --example ex05_doc_tests
```

Open `crates/snapshot-testing/examples/ex05_doc_tests.rs`. It demonstrates:

- Writing functions with doc comments containing code examples
- The `///` syntax for documentation
- How `assert!` and `assert_eq!` work in doc tests
- Running `cargo test --doc` to verify documentation

## What's Next

In [Lesson 09: Modules](./09-modules.md), we'll cover how Rust's module system works -- crates vs modules vs files, visibility with `pub`/`pub(crate)`, the workspace pattern, and why this project splits into 4 crates.
