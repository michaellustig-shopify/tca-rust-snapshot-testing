# Lesson 05: Error Handling

## The Big Idea

Swift has `throws` and `try/catch`. Rust has `Result<T, E>` and the `?` operator. They solve the same problem (handling operations that can fail) but with different philosophies. Swift's errors are more dynamic -- any type conforming to `Error` can be thrown. Rust's errors are more explicit -- the type signature tells you exactly what kinds of errors can occur.

In this project, we use both patterns: `verify_snapshot` returns a `Result` (the testable, composable path), and `assert_snapshot` panics (the convenient, test-friendly path).

## The SnapshotError Type

Open `crates/snapshot-testing/src/assert.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum SnapshotError {
    #[error("Snapshot mismatch:\n{diff}")]
    Mismatch { diff: String },

    #[error("No reference snapshot at {path}. Run with SNAPSHOT_TESTING_RECORD=all to record.")]
    MissingSnapshot { path: PathBuf },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Snapshot timed out after {seconds}s")]
    Timeout { seconds: f64 },
}
```

This is a Rust enum where each variant represents a different failure mode. Compare to Swift:

```swift
// Swift equivalent (hypothetical -- the Swift library returns String?, not a typed error)
enum SnapshotError: Error {
    case mismatch(diff: String)
    case missingSnapshot(path: String)
    case ioError(Error)
    case timeout(seconds: Double)
}
```

### The `thiserror` Crate

The `#[derive(thiserror::Error)]` macro automatically generates implementations of `std::error::Error` and `std::fmt::Display` for each variant. Without it, you'd write:

```rust
// Without thiserror (manual implementation)
impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnapshotError::Mismatch { diff } => write!(f, "Snapshot mismatch:\n{diff}"),
            SnapshotError::MissingSnapshot { path } => write!(
                f, "No reference snapshot at {}. Run with SNAPSHOT_TESTING_RECORD=all to record.",
                path.display()
            ),
            SnapshotError::Io(e) => write!(f, "I/O error: {e}"),
            SnapshotError::Timeout { seconds } => write!(f, "Snapshot timed out after {seconds}s"),
        }
    }
}

impl std::error::Error for SnapshotError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SnapshotError::Io(e) => Some(e),
            _ => None,
        }
    }
}
```

The `#[error("...")]` attribute on each variant defines the `Display` format. The `#[from]` attribute on `Io` generates an automatic conversion from `std::io::Error` into `SnapshotError::Io(...)`.

## `Result<T, E>`: Rust's Error Type

`Result` is an enum with two variants:

```rust
enum Result<T, E> {
    Ok(T),    // success -- contains the value
    Err(E),   // failure -- contains the error
}
```

`verify_snapshot` returns `Result<(), SnapshotError>`:

```rust
pub async fn verify_snapshot<V, F>(
    value: &V,
    snapshotting: &Snapshotting<V, F>,
    name: Option<&str>,
    snapshot_dir: &Path,
    test_name: &str,
) -> Result<(), SnapshotError>
```

`Ok(())` means "the snapshot matched." `Err(...)` means "something went wrong" with details about what.

**Swift comparison:**
```swift
// The Swift library returns String? -- nil means success, Some(message) means failure
func verifySnapshot<Value, Format>(
    of value: Value,
    as snapshotting: Snapshotting<Value, Format>,
    ...
) -> String?
```

The Swift version uses `String?` instead of a typed error. Rust's `Result<(), SnapshotError>` is more structured -- you know exactly which failure modes are possible and can handle them differently.

## The `?` Operator

Look at this code in `verify_snapshot`:

```rust
let data = std::fs::read(&snapshot_path)?;
```

The `?` operator is Rust's equivalent of Swift's `try`. If `fs::read` returns `Err(io_error)`, the `?` immediately returns from the function with `Err(SnapshotError::Io(io_error))`. The conversion from `std::io::Error` to `SnapshotError` happens automatically because of the `#[from]` attribute on the `Io` variant.

Without `?`, you'd write:

```rust
let data = match std::fs::read(&snapshot_path) {
    Ok(data) => data,
    Err(io_error) => return Err(SnapshotError::Io(io_error)),
};
```

**Swift equivalent:**
```swift
let data = try Data(contentsOf: snapshotURL)  // throws if file doesn't exist
```

The `?` operator and `try` do the same thing: early-return on failure.

## WHY verify_snapshot Returns Result, But assert_snapshot Panics

The project has two functions for the same operation:

```rust
// Returns Result -- the caller decides what to do with errors
pub async fn verify_snapshot(...) -> Result<(), SnapshotError> { ... }

// Panics on failure -- convenient for tests
pub async fn assert_snapshot(...) {
    if let Err(e) = verify_snapshot(value, snapshotting, name, snapshot_dir, test_name).await {
        panic!("{e}");
    }
}
```

**WHY two functions?**

1. **`verify_snapshot` is testable.** You can write tests about the testing library itself:
   ```rust
   let result = verify_snapshot(&value, &strategy, None, &dir, "test").await;
   assert!(result.is_err());
   assert!(result.unwrap_err().to_string().contains("Snapshot mismatch"));
   ```
   If it panicked, testing the error case would be awkward (`#[should_panic]` only checks that a panic happened, not what the error was).

2. **`assert_snapshot` is convenient.** In normal test code, you don't want to handle errors -- you want the test to fail immediately with a clear message. Panics do exactly that in Rust's test harness.

**Swift does the same split:**
```swift
// Testable version: returns String? (nil = success, message = failure)
func verifySnapshot(...) -> String?

// Convenient version: calls XCTFail on mismatch
func assertSnapshot(...) {
    if let message = verifySnapshot(...) {
        XCTFail(message)
    }
}
```

## Pattern Matching on Result

The `verify_snapshot` function uses pattern matching to handle different cases:

```rust
match (reference, record) {
    // Reference exists -- compare
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
            Ok(())
        }
    }

    // No reference, never record
    (None, Record::Never) => Err(SnapshotError::MissingSnapshot {
        path: snapshot_path,
    }),

    // No reference, record it
    (None, _) => {
        std::fs::create_dir_all(snapshot_dir)?;
        let data = (snapshotting.diffing.to_data)(&snapshot);
        std::fs::write(&snapshot_path, data)?;
        Ok(())
    }
}
```

Notice how `?` is used throughout -- every `fs::read`, `fs::create_dir_all`, and `fs::write` could fail, and `?` converts those I/O errors into `SnapshotError::Io` automatically.

**Swift equivalent would use `try`:**
```swift
try FileManager.default.createDirectory(at: snapshotDir, ...)
try data.write(to: snapshotPath)
```

## The `#[from]` Attribute

```rust
#[error("I/O error: {0}")]
Io(#[from] std::io::Error),
```

The `#[from]` attribute generates this implementation:

```rust
impl From<std::io::Error> for SnapshotError {
    fn from(error: std::io::Error) -> Self {
        SnapshotError::Io(error)
    }
}
```

This is what makes `?` work on `std::io::Error` inside a function that returns `Result<_, SnapshotError>`. The `?` operator calls `.into()` on the error, which finds this `From` implementation and wraps it.

**Swift analogy:** There's no direct equivalent. Swift's `try` just propagates the error as-is. You'd need a manual `do { ... } catch { throw MyError.io(error) }` to convert error types.

## Panics vs Errors

Rust draws a sharp line between recoverable errors (`Result`) and unrecoverable bugs (`panic!`):

| Mechanism | When to Use | Swift Equivalent |
|-----------|-------------|-----------------|
| `Result<T, E>` | Expected failures (file not found, network error) | `throws` / `try` |
| `panic!()` | Bugs, violated invariants, test failures | `fatalError()` / `preconditionFailure()` |

In this project:

- `verify_snapshot` returns `Result` -- I/O errors and mismatches are expected
- `assert_snapshot` panics -- in test code, a failed assertion should stop the test
- `Record::from_env()` panics on invalid env var values -- an invalid config is a bug

```rust
// From crates/snapshot-testing/src/config.rs
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
```

The panic here is deliberate: if someone sets an environment variable to an unrecognized value, that's a configuration bug that should fail loudly, not silently use a default.

## Exercise

Run the exercise for this lesson:

```bash
cargo run -p snapshot-testing --example ex04_record_modes
```

Open `crates/snapshot-testing/examples/ex04_record_modes.rs`. It walks you through:

- Creating `Result` values and matching on them
- Using the `?` operator
- The difference between returning an error and panicking
- Experimenting with `SnapshotError` variants

## What's Next

In [Lesson 06: Closures](./06-closures.md), we'll dive deep into Rust's closure types (`Fn`, `FnMut`, `FnOnce`), why the snapshot function is `Arc<dyn Fn>` instead of a function pointer, and how `move` semantics work in closures.
