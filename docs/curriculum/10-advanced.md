# Lesson 10: Advanced Topics

## The Big Idea

This lesson covers the patterns that make concurrent, composable test infrastructure possible in Rust: reference counting with `Arc`, interior mutability with `RefCell`, thread-local storage, the `Send` and `Sync` marker traits, and the diff algorithm. These are the tools you reach for when simple ownership isn't enough.

## Arc: Shared Ownership

We've seen `Arc` throughout this project. Let's understand it fully.

In Rust, every value has one owner. When the owner goes out of scope, the value is dropped. But sometimes multiple parts of your code need to share ownership -- nobody knows who will be the last to use it.

`Arc` (Atomic Reference Count) wraps a value in a heap-allocated container with a reference count. Cloning an `Arc` increments the count. Dropping one decrements it. When the count reaches zero, the value is freed.

```rust
use std::sync::Arc;

let a = Arc::new(42);  // ref count: 1
let b = a.clone();     // ref count: 2
let c = a.clone();     // ref count: 3
drop(b);               // ref count: 2
drop(c);               // ref count: 1
drop(a);               // ref count: 0 -> value freed
```

In this project, `Arc` wraps every closure in `Snapshotting` and `Diffing`:

```rust
// From crates/snapshot-testing/src/diffing.rs
pub struct Diffing<V: Clone> {
    pub to_data:   Arc<dyn Fn(&V) -> Vec<u8> + Send + Sync>,
    pub from_data: Arc<dyn Fn(&[u8]) -> V + Send + Sync>,
    pub diff:      Arc<dyn Fn(&V, &V) -> Option<(String, Vec<DiffAttachment>)> + Send + Sync>,
}
```

When you clone a `Diffing`, you get three `Arc::clone()` calls -- three atomic increment operations. The closures themselves are NOT copied. This is cheap.

**Swift comparison:** Swift's ARC (Automatic Reference Counting) does the same thing, but automatically for all reference types (classes, closures). In Rust, you opt in with `Arc`.

### Arc vs Rc

Rust has two reference-counting types:

| Type | Thread Safety | Cost | When to Use |
|------|-------------|------|-------------|
| `Rc<T>` | Single-thread only | Cheaper (non-atomic increments) | When you know the value stays on one thread |
| `Arc<T>` | Thread-safe | Slightly more expensive (atomic increments) | When the value crosses thread boundaries |

We use `Arc` because tests run on multiple threads. `Rc` would cause compile errors because our closures are `Send + Sync`.

## Interior Mutability: RefCell

Look at the thread-local config stack:

```rust
// From crates/snapshot-testing/src/config.rs
thread_local! {
    static CONFIG_STACK: RefCell<Vec<SnapshotTestingConfiguration>> = RefCell::new(Vec::new());
}
```

`RefCell` provides **interior mutability** -- it lets you mutate data even when you only have a shared reference. In Rust, you normally can't mutate through `&` (shared reference). `RefCell` enforces the borrowing rules at runtime instead of compile time:

```rust
CONFIG_STACK.with(|stack| {
    stack.borrow_mut().push(config);  // runtime borrow check
});
```

`borrow_mut()` returns a mutable reference. If something else has already borrowed the `RefCell`, it panics at runtime (instead of a compile error). This is safe because:

1. The thread-local is only accessed within `with()` closures
2. We never hold borrows across await points or function calls
3. The push/pop pattern ensures borrows are very short-lived

**Swift comparison:** Swift doesn't have this concept because Swift classes are always mutable through any reference. `var` properties on a class can be changed by anyone with a reference. Rust's `RefCell` is the opt-in equivalent.

### When to Use What

| Need | Tool | Example in This Project |
|------|------|------------------------|
| Shared ownership | `Arc<T>` | Closures in `Snapshotting` |
| Mutable shared data on one thread | `RefCell<T>` | Config stack |
| Mutable shared data across threads | `Mutex<T>` | Inline snapshot state (architecture doc) |
| Shared ownership + mutation on one thread | `Rc<RefCell<T>>` | Not used (we use thread-locals) |
| Shared ownership + mutation across threads | `Arc<Mutex<T>>` | Inline snapshot state |

## Thread-Local Storage

```rust
// From crates/snapshot-testing/src/config.rs
thread_local! {
    static CONFIG_STACK: RefCell<Vec<SnapshotTestingConfiguration>> = RefCell::new(Vec::new());
}
```

`thread_local!` creates a variable that has a separate instance for each thread. Thread A's config stack is completely independent from Thread B's config stack.

WHY thread-local for config:

1. **Tests run in parallel.** Each `#[test]` function runs on its own thread. If config were global, tests would interfere with each other.

2. **No locking needed.** Since each thread has its own stack, no `Mutex` is needed. Access is fast.

3. **Matches Swift's approach.** Swift uses `@TaskLocal` for the same purpose -- per-task configuration that doesn't leak between tasks.

### How `with_snapshot_testing` Uses Thread-Locals

```rust
pub fn with_snapshot_testing<R, F: FnOnce() -> R>(
    config: SnapshotTestingConfiguration,
    f: F,
) -> R {
    CONFIG_STACK.with(|stack| {
        stack.borrow_mut().push(config);
    });
    let result = f();
    CONFIG_STACK.with(|stack| {
        stack.borrow_mut().pop();
    });
    result
}
```

Pattern: push config, run closure, pop config. The architecture doc shows a more robust version with a `Drop` guard for panic safety:

```rust
// From architecture doc
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

The `Guard` struct ensures the config is popped even if `f()` panics. This is like Swift's `defer`:

```swift
func withSnapshotTesting<R>(config: Config, operation: () throws -> R) rethrows -> R {
    configStack.append(config)
    defer { configStack.removeLast() }
    return try operation()
}
```

## Send and Sync: Thread Safety Markers

Throughout this project, you see `+ Send + Sync` on closure bounds:

```rust
pub snapshot: Arc<dyn Fn(&V) -> Pin<Box<dyn Future<Output = F> + Send>> + Send + Sync>,
```

These are **marker traits** -- they have no methods. They tell the compiler about thread safety:

| Trait | Meaning | Analogy |
|-------|---------|---------|
| `Send` | This type can be transferred to another thread | A value you can mail to someone |
| `Sync` | This type can be accessed from multiple threads simultaneously (via `&`) | A value multiple people can read at once |

Most types are `Send + Sync` automatically. Types that aren't:

- `Rc<T>` -- not `Send` (non-atomic refcount would race)
- `RefCell<T>` -- not `Sync` (runtime borrow checking isn't thread-safe)
- Raw pointers -- not `Send` or `Sync` (no guarantees)

WHY our closures need both:

- **`Send`:** The `Arc` might be moved to another thread (e.g., an async runtime moving a future to a worker thread)
- **`Sync`:** Multiple threads might call the closure simultaneously (tests running in parallel sharing a strategy)

**Swift comparison:** Swift's `Sendable` protocol is equivalent to `Send`. Swift 6's strict concurrency checking enforces similar rules. But Swift doesn't have a `Sync` equivalent -- its actor model handles concurrent access differently.

## The `similar` Crate and Diff Algorithms

The `diff.rs` module wraps the `similar` crate for text diffing:

```rust
// From crates/snapshot-testing/src/diff.rs
use similar::{ChangeTag, TextDiff};

pub fn line_diff(old: &str, new: &str, context_lines: usize) -> Option<String> {
    if old == new {
        return None;
    }

    let diff = TextDiff::from_lines(old, new);
    let mut output = String::new();

    for hunk in diff.unified_diff().context_radius(context_lines).iter_hunks() {
        output.push_str(&format!("{hunk}"));
    }

    if output.is_empty() {
        None
    } else {
        Some(output)
    }
}
```

The `similar` crate uses the **Myers diff algorithm** -- the same algorithm behind `git diff`. It finds the minimum number of insertions and deletions to transform one sequence into another.

WHY we use `similar` instead of porting Swift's diff:

1. The Swift library implements ~200 lines of LCS (Longest Common Subsequence) from scratch
2. `similar` is battle-tested, widely used, and well-maintained
3. It has zero dependencies of its own
4. Myers diff (O(ND) complexity) is optimal for this use case

The `inline_diff` function shows character-level diffing:

```rust
pub fn inline_diff(old: &str, new: &str) -> Option<String> {
    let diff = TextDiff::from_chars(old, new);
    // ... produces inline markers showing which characters differ
}
```

`from_lines` vs `from_chars` -- same algorithm, different granularity. Lines for multi-line comparisons, characters for single-line comparisons.

## Unsafe: When You'd Need It, Why We Don't

`unsafe` blocks let you bypass Rust's safety guarantees. Common uses:

1. Calling C code (FFI)
2. Dereferencing raw pointers
3. Implementing unsafe traits (`Send`, `Sync` manually)

**This project doesn't use `unsafe` at all.** The types we use (`Arc`, `Box`, `Vec`, `String`, `RefCell`) handle all the low-level memory management safely. The `similar` crate also avoids `unsafe`.

If this were a performance-critical library (not test code), you might consider `unsafe` for:
- Custom allocators for snapshot data
- Lock-free data structures for the config stack
- SIMD-accelerated binary diff

But for test infrastructure, safe Rust is more than fast enough, and correctness matters more than performance.

## Exercise

Run the exercise for this lesson:

```bash
cargo run -p snapshot-testing --example ex04_record_modes
```

This exercise demonstrates:

- Thread-local configuration with `with_snapshot_testing`
- Nesting configurations (inner overrides outer)
- How `current_record()` walks the stack
- The `RefCell` borrow pattern

## Putting It All Together

Across these 10 lessons, you've seen how Rust's type system enables a composable, safe, and efficient snapshot testing library:

| Concept | Where It Appears | WHY |
|---------|-----------------|-----|
| Ownership/borrowing | `verify_snapshot(&value, ...)` | Don't consume test values |
| Arc | `Arc<dyn Fn>` closures | Shared, cloneable strategies |
| Generics | `Snapshotting<V, F>` | One library, any type |
| Traits | `Debug`, `Clone`, `Send`, `Sync` derives | Standard behavior for types |
| Closures | Snapshot functions, diff functions | Composable behavior |
| Futures | `Pin<Box<dyn Future>>` | Async snapshot support |
| Error handling | `Result<(), SnapshotError>` | Testable error paths |
| Thread-locals | `CONFIG_STACK` | Per-test configuration |
| Modules | 4 crates, internal modules | Clean dependency boundaries |

The architecture mirrors Swift's Point-Free library closely, but uses Rust-specific patterns where Swift's automatic memory management doesn't apply. The result is a library that's explicit about its costs, safe by construction, and just as composable as the Swift original.
