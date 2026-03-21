# Architecture Document

## rust-snapshot-testing v1.0.0

---

## Table of Contents

1. [Module Dependency Graph](#1-module-dependency-graph)
2. [Core Type Relationships](#2-core-type-relationships)
3. [Trait Hierarchy](#3-trait-hierarchy)
4. [Lifetime and Ownership Strategy](#4-lifetime-and-ownership-strategy)
5. [Concurrency Model](#5-concurrency-model)
6. [Memory Management](#6-memory-management)
7. [Configuration Threading Model](#7-configuration-threading-model)
8. [Snapshot Execution Flow](#8-snapshot-execution-flow)
9. [Inline Snapshot Architecture](#9-inline-snapshot-architecture)
10. [Key Design Decisions](#10-key-design-decisions)

---

## 1. Module Dependency Graph

### Crate-Level Dependencies

```
                    ┌─────────────────────────┐
                    │     User Test Code      │
                    └──────┬──────────────────┘
                           │ uses
              ┌────────────┼────────────────┐
              │            │                │
              ▼            ▼                ▼
┌─────────────────┐ ┌──────────────┐ ┌──────────────────────┐
│ snapshot-testing │ │   inline-    │ │ snapshot-testing-    │
│   (core crate)  │ │  snapshot-   │ │    custom-dump       │
│                 │ │   testing    │ │                      │
└────────┬────────┘ └──────┬───────┘ └──────────┬───────────┘
         │                 │                     │
         │                 │ depends on          │ depends on
         │                 ▼                     ▼
         │          ┌──────────────┐     ┌──────────────┐
         │          │  snapshot-   │     │  snapshot-   │
         │          │   testing    │     │   testing    │
         │          └──────────────┘     └──────────────┘
         │
         │ uses (external)
         ▼
   ┌───────────┐  ┌───────────┐  ┌───────────────┐
   │  similar   │  │ thiserror │  │  serde_json   │
   │ (diffing)  │  │ (errors)  │  │ (json strat)  │
   └───────────┘  └───────────┘  └───────────────┘
```

### Internal Module Graph (snapshot-testing crate)

```
                    lib.rs
                    (re-exports)
                       │
          ┌────────────┼────────────────┐
          │            │                │
          ▼            ▼                ▼
    ┌──────────┐ ┌──────────┐    ┌──────────┐
    │ assert.rs│ │config.rs │    │strategies│
    │          │ │          │    │  /mod.rs  │
    └────┬─────┘ └────┬─────┘    └────┬─────┘
         │            │               │
         │uses        │               │uses
         ▼            │               ▼
    ┌────────────┐    │         ┌──────────────┐
    │snapshotting│◄───┘         │ snapshotting  │
    │    .rs     │              │   (pullback)  │
    └────┬───────┘              └──────┬───────┘
         │                             │
         │contains                     │uses
         ▼                             ▼
    ┌──────────┐                ┌──────────┐
    │diffing.rs│                │ diff.rs  │
    │          │◄───────────────│          │
    └──────────┘   used by      └──────────┘
                   Diffing::lines()
```

### Key: Who depends on whom

| Module | Depends On |
|--------|-----------|
| `lib.rs` | all modules (re-exports) |
| `assert.rs` | `snapshotting`, `config`, `diffing` |
| `config.rs` | nothing (leaf module) |
| `snapshotting.rs` | `diffing` |
| `diffing.rs` | nothing (leaf module) |
| `diff.rs` | `similar` (external crate) |
| `strategies/mod.rs` | `snapshotting`, `diffing`, `diff` |

---

## 2. Core Type Relationships

```
┌─────────────────────────────────────────────────────────────────┐
│                         Snapshotting<V, F>                      │
│  ┌─────────────────────┐                                        │
│  │ path_extension:     │                                        │
│  │   Option<String>    │                                        │
│  ├─────────────────────┤                                        │
│  │ diffing:            │──────────►  Diffing<F>                 │
│  │   Diffing<F>        │           ┌────────────────────┐       │
│  ├─────────────────────┤           │ to_data:  &F→Vec   │       │
│  │ snapshot:           │           │ from_data: &[u8]→F │       │
│  │   Arc<Fn(&V)→Fut<F>>│           │ diff: (&F,&F)→Opt  │       │
│  └─────────────────────┘           │   ↓                │       │
│                                    │ (String,            │       │
│  pullback(Fn(&NV)->V)              │  Vec<DiffAttach>)  │       │
│       │                            └────────────────────┘       │
│       ▼                                                         │
│  Snapshotting<NV, F>               DiffAttachment               │
│  (new strategy, same diffing)      ┌─────────────────┐         │
│                                    │ Data {           │         │
│                                    │   bytes: Vec<u8> │         │
│                                    │   name: String   │         │
│                                    │ }                │         │
│                                    └─────────────────┘         │
└─────────────────────────────────────────────────────────────────┘
```

### How pullback composes strategies

```
Snapshotting<String, String>    ← base strategy (.lines)
        │
        │ pullback(Display::to_string)
        ▼
Snapshotting<V, String>         ← .description strategy
        │
        │ pullback(|req: &Request| format_request(req))
        ▼
Snapshotting<Request, String>   ← .raw strategy for HTTP requests
```

The critical insight: **pullback goes backwards**. To create a strategy for `Request`, you start with a strategy for `String` and provide a function `Request -> String`. The chain can go as deep as needed.

---

## 3. Trait Hierarchy

### 3.1 No Core Trait Required

Unlike many Rust testing libraries, `rust-snapshot-testing` does NOT require types to implement a trait to be snapshot-tested. Instead, strategies are external values. This means:

- Any type can be snapshot-tested (even types from other crates).
- The same type can have multiple strategies.
- Strategies compose via pullback.

### 3.2 Traits Used for Built-in Strategies

```
                    ┌─────────────────┐
                    │  std::fmt::Debug │
                    │  (Rust built-in) │
                    └────────┬────────┘
                             │ enables
                             ▼
                    Snapshotting::debug()
                    Snapshotting::custom_dump()

                    ┌───────────────────┐
                    │ std::fmt::Display  │
                    │  (Rust built-in)   │
                    └────────┬──────────┘
                             │ enables
                             ▼
                    Snapshotting::description()

                    ┌───────────────────┐
                    │ serde::Serialize   │
                    │  (serde crate)     │
                    └────────┬──────────┘
                             │ enables
                             ▼
                    Snapshotting::json()
```

### 3.3 Optional Trait: `SnapshotDisplay`

```rust
pub trait SnapshotDisplay {
    fn snapshot_description(&self) -> String;
    fn render_children() -> bool { false }
}
```

Swift equivalent: `AnySnapshotStringConvertible`

This trait is entirely optional. It provides a hook for types to customize their appearance in the `custom_dump` strategy. Types that don't implement it fall back to `Debug`.

### 3.4 Marker Traits on Core Types

| Type | Traits | Why |
|------|--------|-----|
| `Snapshotting<V, F>` | `Clone`, `Debug` | Shared across test assertions; diagnostics |
| `Diffing<V>` | `Clone`, `Debug` | Stored inside Snapshotting, may be shared |
| `DiffAttachment` | `Debug`, `Clone` | Part of error output |
| `Record` | `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq` | Value type, used in comparisons |
| `DiffTool` | `Clone`, `Debug` | Stored in config, may be shared |
| `SnapshotTestingConfiguration` | `Debug`, `Clone`, `Default` | Stored in thread-local stack |
| `SnapshotError` | `Debug`, `Display` (via thiserror) | Error type |

---

## 4. Lifetime and Ownership Strategy

### 4.1 Principle: Own Everything, Share via Arc

The Swift library uses closures freely because Swift has ARC (automatic reference counting). In Rust, we replicate this with `Arc<dyn Fn(...)>` for all closures stored in structs.

```
Swift:  var snapshot: (Value) -> Async<Format>
Rust:   snapshot: Arc<dyn Fn(&V) -> Pin<Box<dyn Future<Output = F> + Send>> + Send + Sync>
```

**Why Arc, not Box?** `Box<dyn Fn>` is not `Clone`. Since `Snapshotting` needs to be cloneable (strategies are often shared or pulled back multiple times), we need `Arc`.

### 4.2 References vs Owned Values

| Context | Swift | Rust | Rationale |
|---------|-------|------|-----------|
| Value passed to snapshot | `(Value) -> ...` (owned/copied) | `(&V) -> ...` (borrowed) | Avoid cloning large values |
| Value passed to diff | `(Value, Value) -> ...` | `(&V, &V) -> ...` | Comparison never needs ownership |
| Value from from_data | `(Data) -> Value` | `(&[u8]) -> V` | Deserialize produces owned value from borrowed bytes |
| Value to to_data | `(Value) -> Data` | `(&V) -> Vec<u8>` | Serialize borrows value, produces owned bytes |
| Transform in pullback | `(NewValue) -> Value` | `Fn(&NewV) -> V` | Borrow new value, produce owned old value |

### 4.3 Where Lifetimes Appear

Lifetimes are NOT needed on the core types. All closures are `'static` (owned by Arc). All stored data is owned. This is intentional -- it keeps the API simple and avoids lifetime parameter proliferation.

The only place borrows appear is in function parameters:

```rust
verify_snapshot(
    value: &V,                    // borrowed
    snapshotting: &Snapshotting,  // borrowed
    snapshot_dir: &Path,          // borrowed
    test_name: &str,              // borrowed
)
```

### 4.4 The `'static` Bound

`Snapshotting<V, F>` requires `V: 'static` and `F: 'static`. This means the value and format types cannot contain non-static references. This is acceptable because:

1. Snapshot values are typically structs, enums, strings, or byte arrays.
2. The `'static` bound comes from the `Arc<dyn Fn>` closures which must own their captures.
3. Swift has the same constraint implicitly -- closures capture values, not references.

---

## 5. Concurrency Model

### 5.1 Swift Async to Rust Futures

```
Swift:                              Rust:
┌────────────────────┐              ┌────────────────────────────────┐
│ Async<Value>       │              │ Pin<Box<dyn Future<Output=V>>> │
│ ├ run: (cb) -> ()  │    ────►     │                                │
│ └ init(value: V)   │              │ std::future::ready(v)          │
└────────────────────┘              └────────────────────────────────┘
```

Swift's `Async<Value>` is a callback-based wrapper (not Swift concurrency `async/await`). It wraps a function that calls a callback with the result. In Rust, we use standard `Future`s instead because:

1. Rust has first-class async/await.
2. Callback-based patterns are unidiomatic in Rust.
3. Futures compose better (`.map`, `.and_then`, timeout).

### 5.2 Timeout Handling

```
Swift:                              Rust:
┌────────────────────┐              ┌──────────────────────────┐
│ XCTestExpectation   │              │ tokio::time::timeout(    │
│ XCTWaiter.wait(     │    ────►     │   duration,              │
│   for: [exp],       │              │   snapshot_future        │
│   timeout: 5        │              │ )                        │
│ )                   │              └──────────────────────────┘
└────────────────────┘
```

### 5.3 Runtime Agnosticism

The core library should work with any async runtime. Strategy:

```
┌──────────────────────────────────────┐
│ Feature flags:                        │
│                                       │
│  [features]                           │
│  default = []                         │
│  tokio = ["dep:tokio"]                │
│  async-std = ["dep:async-std"]        │
│                                       │
│  When neither is enabled:             │
│  - verify_snapshot is sync            │
│  - Async strategies are not available │
│  - Sync strategies still work         │
└──────────────────────────────────────┘
```

For v1.0.0, we target `tokio` as the primary runtime since it is the most widely used. The `verify_snapshot` function is async. Tests must use `#[tokio::test]`.

---

## 6. Memory Management

### 6.1 Swift ARC to Rust Arc

```
Swift (ARC, automatic):
┌──────────────┐     ┌──────────────┐
│ Snapshotting  │     │ Snapshotting  │
│ .snapshot ────┼─RC──│ .snapshot ────│  (two refs to same closure)
└──────────────┘     └──────────────┘

Rust (Arc, explicit):
┌──────────────┐     ┌──────────────┐
│ Snapshotting  │     │ Snapshotting  │
│ .snapshot ────┼─Arc─│ .snapshot ────│  (Arc::clone is cheap)
└──────────────┘     └──────────────┘
```

### 6.2 Closure Capture Strategy

```rust
// BAD: Would require V to be Clone
snapshot: Box<dyn Fn(V) -> F>  // takes ownership each call

// GOOD: Borrows the value
snapshot: Arc<dyn Fn(&V) -> Pin<Box<dyn Future<Output = F> + Send>>>
```

By taking `&V` instead of `V`, we avoid requiring `Clone` on the value type. The snapshot function borrows the value, processes it, and returns an owned format value.

### 6.3 No Cyclic References

The type graph is acyclic:
- `Snapshotting` contains `Diffing` (not the other way around).
- `pullback` creates a new `Snapshotting` that captures the old one's `snapshot` closure, but this is a unidirectional chain, not a cycle.

---

## 7. Configuration Threading Model

```
┌─────────────────────────────────────────────────────────────────┐
│ Thread 1 (test_foo)            Thread 2 (test_bar)              │
│                                                                  │
│ TLS: [Record::All]             TLS: []  ← empty, uses default  │
│                                                                  │
│ with_snapshot_testing(          verify_snapshot(...)             │
│   Record::All,                   → reads TLS → empty            │
│   || {                           → reads env var                │
│     verify_snapshot(...)         → falls back to Missing        │
│       → reads TLS → All                                         │
│   }                                                              │
│ )                                                                │
│ ← TLS popped back to []                                         │
└─────────────────────────────────────────────────────────────────┘
```

### Resolution Stack

```
with_snapshot_testing(config_A, || {
    with_snapshot_testing(config_B, || {
        // TLS stack: [config_A, config_B]
        // current_record() walks stack from top:
        //   config_B.record → if Some, use it
        //   config_A.record → if Some, use it
        //   env var → if set, use it
        //   default → Missing
    });
    // TLS stack: [config_A]
});
// TLS stack: []
```

### Drop Guard for Panic Safety

```rust
pub fn with_snapshot_testing<R>(config: SnapshotTestingConfiguration, f: impl FnOnce() -> R) -> R {
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            CONFIG_STACK.with(|stack| { stack.borrow_mut().pop(); });
        }
    }

    CONFIG_STACK.with(|stack| { stack.borrow_mut().push(config); });
    let _guard = Guard;  // popped on drop, even if f() panics
    f()
}
```

---

## 8. Snapshot Execution Flow

### 8.1 Full verify_snapshot Flow

```
verify_snapshot(value, snapshotting, name, record, snapshot_dir, test_name, timeout)
  │
  ├─ 1. Generate snapshot
  │     snapshot_future = (snapshotting.snapshot)(&value)
  │     snapshot = timeout(duration, snapshot_future).await?
  │           │
  │           ├── Ok(format_value) → continue
  │           └── Err(Elapsed) → return Err(Timeout)
  │
  ├─ 2. Resolve record mode
  │     record = explicit_param
  │           ?? thread_local_config
  │           ?? env_var
  │           ?? Record::Missing
  │
  ├─ 3. Build file path
  │     dir = snapshot_dir  (or derive from source file)
  │     name_part = name ?? counter.next()
  │     sanitized_test = sanitize(test_name)
  │     ext = snapshotting.path_extension ?? "txt"
  │     path = dir / "{sanitized_test}.{name_part}.{ext}"
  │
  ├─ 4. Branch on record mode
  │     ├── Record::All
  │     │     → write snapshot to path
  │     │     → return Err(Recorded { path })
  │     │
  │     ├── path does NOT exist
  │     │     ├── Record::Never → return Err(MissingSnapshot)
  │     │     └── otherwise → write snapshot, return Err(Recorded)
  │     │
  │     └── path EXISTS
  │           → reference = (diffing.from_data)(read(path))
  │           → diff_result = (diffing.diff)(&reference, &snapshot)
  │           │
  │           ├── None (match) → return Ok(())
  │           │
  │           └── Some((message, attachments))
  │                 ├── Record::Failed → write, return Err(Mismatch)
  │                 └── otherwise → write artifacts, return Err(Mismatch)
  │
  └─ 5. On mismatch: write artifacts
        artifact_dir = env(SNAPSHOT_ARTIFACTS) ?? temp_dir()
        artifact_path = artifact_dir / file_stem / snapshot_filename
        write(artifact_path, to_data(snapshot))
        diff_cmd = diff_tool.command(path, artifact_path)
```

### 8.2 assert_snapshot Macro Expansion

```rust
// User writes:
assert_snapshot!(my_value, as: Snapshotting::json());

// Macro expands to approximately:
{
    let __snapshot_file = file!();
    let __snapshot_line = line!();
    let __snapshot_dir = __derive_snapshot_dir(__snapshot_file);
    let __test_name = __derive_test_name(module_path!());

    let result = verify_snapshot(
        &my_value,
        &Snapshotting::json(),
        None,          // name
        None,          // record (use config)
        &__snapshot_dir,
        &__test_name,
        Duration::from_secs(5),
    ).await;

    if let Err(e) = result {
        panic!("{}:{}: {}", __snapshot_file, __snapshot_line, e);
    }
}
```

---

## 9. Inline Snapshot Architecture

### 9.1 Two-Phase Execution

```
Phase 1: During test execution
┌──────────────────────────────────────────────┐
│ assert_inline_snapshot!(value, @"expected")   │
│   │                                          │
│   ├── Generate snapshot                      │
│   ├── Compare with "expected"                │
│   ├── If mismatch AND recording:             │
│   │     → Stash (file, line, actual) in      │
│   │       global INLINE_SNAPSHOT_STATE       │
│   ├── Report test failure                    │
│   └── Continue (do NOT rewrite yet)          │
└──────────────────────────────────────────────┘

Phase 2: At process exit (atexit hook)
┌──────────────────────────────────────────────┐
│ For each file in INLINE_SNAPSHOT_STATE:       │
│   │                                          │
│   ├── Parse source file with syn              │
│   ├── Find macro invocations by line number  │
│   ├── Replace expected string literals       │
│   └── Write updated source atomically        │
└──────────────────────────────────────────────┘
```

### 9.2 Global State for Inline Snapshots

```rust
static INLINE_SNAPSHOT_STATE: Mutex<HashMap<PathBuf, Vec<InlineSnapshot>>> =
    Mutex::new(HashMap::new());

struct InlineSnapshot {
    expected: Option<String>,
    actual: Option<String>,
    was_recording: bool,
    function: String,
    line: u32,
    column: u32,
}
```

This must be a global (not thread-local) because:
1. Multiple test threads may write inline snapshots to the same file.
2. The atexit handler runs once, in a single thread, and needs all data.
3. Access is synchronized via `Mutex`.

---

## 10. Key Design Decisions

### 10.1 Struct-based strategies, not traits

**Decision**: Use `Snapshotting<V, F>` as a struct with closures, not a `trait Snapshot { ... }`.

**Why**: Traits would mean one strategy per type (without newtype wrappers). Structs allow multiple strategies for the same type. This matches Swift's design exactly and is the core insight of Point-Free's architecture.

**Tradeoff**: Closures in structs require `Arc<dyn Fn>` which has a small runtime cost. This is negligible for test code.

### 10.2 Arc closures, not generics

**Decision**: Store closures as `Arc<dyn Fn>` rather than making `Diffing` and `Snapshotting` generic over the closure types.

**Why**: If we used generics, `Diffing<V, TD, FD, D>` would have three type parameters just for the closures. `Snapshotting` would inherit those plus its own. Type signatures would be unwieldy. `Arc<dyn Fn>` keeps the API clean while enabling `Clone`.

**Tradeoff**: Dynamic dispatch for closure calls. Negligible for test code.

### 10.3 Futures instead of callbacks

**Decision**: Use `Pin<Box<dyn Future>>` instead of Swift's callback-based `Async<Value>`.

**Why**: Rust has native async/await. Callback-based patterns are unidiomatic and harder to compose. Futures give us timeout support, `.map`, combinators, and integration with async runtimes for free.

**Tradeoff**: Requires an async runtime for async strategies. Sync strategies can use `std::future::ready()` which resolves immediately.

### 10.4 Thread-local config, not task-local

**Decision**: Use `thread_local!` for the configuration stack.

**Why**: Swift uses `@TaskLocal` because Swift concurrency is task-based. Rust tests run on threads (even with tokio, each `#[tokio::test]` gets its own thread by default). Thread-local gives each test its own config without races.

**Tradeoff**: If a single test spawns multiple tasks that cross thread boundaries, the config won't propagate. This matches Swift's behavior -- `@TaskLocal` also doesn't propagate to detached tasks.

### 10.5 `&V` not `V` in snapshot function

**Decision**: The snapshot function takes `&V` (a reference) rather than `V` (owned value).

**Why**: Avoids requiring `Clone` on every value type. In tests, you typically want to snapshot a value and then continue using it. Taking ownership would force the user to clone.

**Tradeoff**: The snapshot function cannot consume the value. If a transformation needs ownership, it must clone internally. This is rare in practice.

### 10.6 `similar` crate, not hand-rolled diff

**Decision**: Use the `similar` crate for text diffing instead of porting Swift's LCS algorithm.

**Why**: The Swift library implements a ~200-line LCS-based diff from scratch. The `similar` crate provides a battle-tested Myers diff implementation with O(ND) complexity, the same algorithm used by `git diff`. No reason to reimplement.

**Tradeoff**: External dependency. But `similar` is well-maintained, has zero dependencies of its own, and is widely used in the Rust ecosystem.

### 10.7 Macro for assert, function for verify

**Decision**: `assert_snapshot!` is a macro; `verify_snapshot` is a function.

**Why**: The macro captures `file!()`, `line!()`, and `module_path!()` automatically -- these are only available as macros. `verify_snapshot` is a plain function for testability (returns `Result`, no panicking, no magic).

**Tradeoff**: Macros are harder to debug. But the macro is thin -- it just captures location info and delegates to `verify_snapshot`.

### 10.8 Panics for assertion failures (not Result)

**Decision**: `assert_snapshot!` panics on failure (like `assert!` and `assert_eq!`).

**Why**: Rust test failures are panics. This integrates with the standard test harness, `#[should_panic]`, and test output formatting. `verify_snapshot` is the non-panicking alternative for custom wrappers.

### 10.9 Recording returns an error, not Ok

**Decision**: When a snapshot is recorded (new or re-recorded), `verify_snapshot` returns `Err(Recorded)`, not `Ok(())`.

**Why**: This matches Swift's behavior where recording always triggers a test "failure" (an XCTFail message). The user must re-run the test to verify the recorded snapshot. This prevents accidentally committing unreviewed snapshots.

**Tradeoff**: The first run of any new snapshot test always "fails". This is intentional -- it forces the developer to inspect the snapshot.
