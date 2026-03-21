# Lesson 09: Modules, Crates, and the Workspace

## The Big Idea

Swift has modules (targets in SPM) and files within modules. Rust has a three-level hierarchy: **workspace** (like an SPM package with multiple targets), **crates** (like individual SPM targets), and **modules** (like files/directories within a target). Understanding this hierarchy is key to navigating any Rust project.

## The Three Levels

| Rust Term | Swift/SPM Equivalent | Example in This Project |
|-----------|---------------------|------------------------|
| **Workspace** | Package (Package.swift) | The root `Cargo.toml` with `[workspace]` |
| **Crate** | Target | `snapshot-testing`, `inline-snapshot-testing`, etc. |
| **Module** | File or directory within a target | `snapshotting`, `diffing`, `config` |

## Level 1: The Workspace

The root `Cargo.toml` defines the workspace:

```toml
# From Cargo.toml (root)
[workspace]
resolver = "2"
members = [
    "crates/snapshot-testing",
    "crates/inline-snapshot-testing",
    "crates/snapshot-testing-custom-dump",
    "crates/trinity",
]
```

This is like a Swift `Package.swift` with multiple targets:

```swift
// Swift equivalent (conceptual)
let package = Package(
    name: "rust-snapshot-testing",
    targets: [
        .target(name: "SnapshotTesting", ...),
        .target(name: "InlineSnapshotTesting", ...),
        .target(name: "SnapshotTestingCustomDump", ...),
        .executableTarget(name: "Trinity", ...),
    ]
)
```

### Workspace Dependencies

Shared dependency versions are defined once:

```toml
[workspace.dependencies]
similar = "2"
serde = { version = "1", features = ["derive"] }
thiserror = "2"
tokio = { version = "1", features = ["full"] }
```

Individual crates reference them:

```toml
# From crates/snapshot-testing/Cargo.toml
[dependencies]
similar = { workspace = true }
thiserror = { workspace = true }
```

**Swift equivalent:**
```swift
// Swift doesn't have workspace-level dependency sharing.
// Each target declares its own dependencies, though packages resolve once.
```

## WHY We Split Into 4 Crates

This maps directly to Swift's 3 modules plus our CLI tool:

| Crate | Swift Module | WHY Separate |
|-------|-------------|--------------|
| `snapshot-testing` | `SnapshotTesting` | Core engine. Most users only need this. |
| `inline-snapshot-testing` | `InlineSnapshotTesting` | Needs `syn` (a Rust parser) to rewrite source files. Heavy dependency that core users don't need. |
| `snapshot-testing-custom-dump` | `SnapshotTestingCustomDump` | Custom pretty-printing. Optional feature, not everyone needs it. |
| `trinity` | *(no equivalent)* | CLI tool for project management. Not a library at all. |

**The principle:** Each crate should have a clear purpose and minimal dependencies. A user who just wants basic snapshot testing shouldn't have to compile a Rust parser (`syn`).

**Swift applies the same principle:**
```swift
// Swift: You import only what you need
import SnapshotTesting           // core
import InlineSnapshotTesting     // only if you need inline snapshots
```

## Level 2: Crates and Their Cargo.toml

Each crate has its own `Cargo.toml`:

```toml
# From crates/snapshot-testing/Cargo.toml
[package]
name = "snapshot-testing"
version.workspace = true
edition.workspace = true

[dependencies]
similar = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
insta = { workspace = true }
```

Key sections:

- `[dependencies]` -- what the crate needs at runtime
- `[dev-dependencies]` -- what it needs only for tests (not shipped to users)

**Swift equivalent:**
```swift
.target(
    name: "SnapshotTesting",
    dependencies: [
        .product(name: "Similar", package: "similar"),
    ]
)
```

## Level 3: Modules Within a Crate

Inside `crates/snapshot-testing/src/`, modules are defined by files:

```
src/
  lib.rs            -- crate root, declares modules
  snapshotting.rs   -- the `snapshotting` module
  diffing.rs        -- the `diffing` module
  diff.rs           -- the `diff` module
  assert.rs         -- the `assert` module
  config.rs         -- the `config` module
```

The crate root (`lib.rs`) declares and re-exports:

```rust
// From crates/snapshot-testing/src/lib.rs
pub mod assert;
pub mod config;
pub mod diff;
pub mod diffing;
pub mod snapshotting;

// Re-export key types at crate root
pub use assert::{assert_snapshot, verify_snapshot};
pub use config::{DiffTool, Record, SnapshotTestingConfiguration};
pub use diff::line_diff;
pub use diffing::Diffing;
pub use snapshotting::Snapshotting;
```

### `pub mod` vs `mod`

- `pub mod config;` -- the module is public. External crates can access it: `snapshot_testing::config::Record`
- `mod internal;` -- the module is private. Only this crate can see it.

### `pub use` -- Re-exports

```rust
pub use config::{DiffTool, Record, SnapshotTestingConfiguration};
```

This re-exports items at the crate root. Users can write:
```rust
use snapshot_testing::Record;  // short path
```

Instead of:
```rust
use snapshot_testing::config::Record;  // full path (also works)
```

**Swift equivalent:**
```swift
// Swift: @_exported import re-exports an entire module
// Individual re-exports aren't needed -- Swift's access control is different
```

## Visibility: `pub`, `pub(crate)`, `pub(super)`, private

| Rust | Swift Equivalent | Meaning |
|------|-----------------|---------|
| `pub` | `public` | Visible to everyone |
| `pub(crate)` | `internal` (default) | Visible within the crate only |
| `pub(super)` | *(no equivalent)* | Visible to the parent module only |
| *(no keyword)* | `private` | Visible within the current module only |

In this project, most items are `pub` because they're the public API:

```rust
// Public struct -- part of the API
pub struct Snapshotting<V, F> { ... }

// Public fields -- users can access them directly
pub path_extension: Option<String>,
pub diffing: Diffing<F>,
pub snapshot: Arc<dyn Fn(&V) -> ...>,
```

**Swift comparison:**
```swift
// Swift: Same fields would be
public var pathExtension: String?
public var diffing: Diffing<Format>
public var snapshot: (Value) -> Async<Format>
```

The biggest difference: **Rust defaults to private.** If you forget `pub`, nothing outside the module can see it. Swift defaults to `internal` (visible within the module/target).

## How Modules Reference Each Other

Within the same crate, modules use `crate::` paths:

```rust
// From crates/snapshot-testing/src/snapshotting.rs
use crate::diffing::Diffing;
```

`crate::` means "start from the root of this crate." It's like Swift's module name:

```swift
// Swift: Within SnapshotTesting module
import struct SnapshotTesting.Diffing  // rarely needed, usually just use the type
```

Between crates in the workspace:

```rust
// From crates/inline-snapshot-testing, referencing snapshot-testing
use snapshot_testing::Snapshotting;
```

This is like Swift's cross-module imports:
```swift
import SnapshotTesting
```

## The `tests/` Directory

Integration tests live in a separate directory:

```
crates/snapshot-testing/
  src/           -- library source
  tests/         -- integration tests
    helpers.rs
    snapshot_testing_tests.rs
    record_tests.rs
    ...
```

Each file in `tests/` is compiled as its own crate. It can only use the public API:

```rust
// From crates/snapshot-testing/tests/snapshot_testing_tests.rs
use snapshot_testing::{
    assert_snapshot, verify_snapshot, Diffing, Record, SnapshotTestingConfiguration,
    Snapshotting,
};
```

**Swift equivalent:** Test targets in SPM:
```swift
.testTarget(
    name: "SnapshotTestingTests",
    dependencies: ["SnapshotTesting"],
    path: "Tests/SnapshotTestingTests"
)
```

## The Module File vs Directory Pattern

Rust modules can be single files or directories:

```
// Single file module
src/config.rs       -- module `config`

// Directory module (for when you need sub-modules)
src/strategies/
  mod.rs            -- module `strategies` (the "index" file)
  json.rs           -- sub-module `strategies::json`
  debug.rs          -- sub-module `strategies::debug`
```

`mod.rs` in a directory is like an `index.swift` -- it defines what the module exports. The architecture doc mentions a `strategies/` directory for built-in strategies, which would follow this pattern.

## Exercise

There's no standalone exercise for this lesson, but you can explore the module structure:

```bash
# See the workspace members
cargo metadata --format-version 1 | python3 -m json.tool | grep -A2 '"name"'

# Build just one crate
cargo build -p snapshot-testing

# Run tests for just one crate
cargo test -p snapshot-testing

# See the dependency tree
cargo tree -p snapshot-testing
```

Also look at how `lib.rs` re-exports items and how test files import them.

## What's Next

In [Lesson 10: Advanced Topics](./10-advanced.md), we'll cover the trickier patterns in this project: `Arc` with interior mutability, thread-local storage (`thread_local!`), `Send + Sync` traits, and when you might (or might not) need `unsafe`.
