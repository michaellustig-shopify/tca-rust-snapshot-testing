# Lesson 07: Async Rust

## The Big Idea

Both Swift and Rust have async/await, but the underlying machinery is very different. Swift's async is built into the runtime with a cooperative thread pool. Rust's async is a zero-cost abstraction -- futures are state machines compiled at build time, and you bring your own runtime (like `tokio`). This lesson explains why the snapshot function returns `Pin<Box<dyn Future>>` and how async works in this project.

## Swift's Async<Value> vs Rust's Future

The Swift snapshot testing library doesn't use Swift concurrency (`async/await`). It uses its own `Async<Value>` type, which is a callback wrapper:

```swift
// Swift's custom Async type (NOT Swift concurrency)
struct Async<Value> {
    let run: (@escaping (Value) -> Void) -> Void
}

// Usage:
let async = Async<String> { callback in
    callback("hello")  // call back with the value
}
async.run { value in
    print(value)  // "hello"
}
```

In Rust, we replace this with standard `Future`:

```rust
// Rust: Standard Future trait
trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
```

You usually don't implement `Future` by hand -- you write `async` functions or blocks:

```rust
async fn compute_snapshot(value: &str) -> String {
    format!("Snapshot: {}", value)
}
```

The compiler turns this into a state machine that implements `Future<Output = String>`.

## The Snapshot Closure Type, Decoded

Here's the full type from `Snapshotting`:

```rust
pub snapshot: Arc<dyn Fn(&V) -> Pin<Box<dyn Future<Output = F> + Send>> + Send + Sync>,
```

Let's read this inside-out:

| Part | Meaning |
|------|---------|
| `Future<Output = F>` | An async computation that produces an `F` value |
| `dyn Future<...>` | Type-erased future (could be any future type) |
| `+ Send` | The future can be sent between threads |
| `Box<dyn Future<...>>` | Heap-allocated, owned, type-erased future |
| `Pin<Box<...>>` | The future is "pinned" in memory (can't be moved) |
| `Fn(&V) -> Pin<Box<...>>` | A function that takes a `&V` and returns such a future |
| `dyn Fn(...)` | Type-erased function (closure) |
| `+ Send + Sync` | The closure is thread-safe |
| `Arc<dyn Fn(...)>` | Reference-counted, cloneable, thread-safe closure |

**Swift equivalent:**
```swift
var snapshot: (Value) -> Async<Format>
```

Swift's version is much shorter because Swift handles type erasure, reference counting, and memory management automatically.

## WHY `Pin<Box<dyn Future>>`

Three problems solved by three layers:

### Why `dyn Future`? (Type Erasure)

Every `async` block has a unique, unnameable type (just like closures -- see [Lesson 06](./06-closures.md)). When you write:

```rust
async { "hello".to_string() }
```

The compiler generates a struct like `__AsyncBlock47` that implements `Future<Output = String>`. You can't write `__AsyncBlock47` in your code.

Since `Snapshotting` needs to store any async computation (not a specific one), we use `dyn Future` to erase the concrete type.

### Why `Box`? (Heap Allocation)

`dyn Future` is a trait object -- its size is unknown at compile time. Rust requires all struct fields to have known sizes. `Box` puts the future on the heap and stores a fixed-size pointer.

### Why `Pin`? (Memory Safety for Futures)

This is the trickiest part. Some futures contain self-references (pointers to their own fields). If you move such a future in memory, those internal pointers would become invalid.

`Pin` guarantees that the future won't be moved after creation. The async runtime (tokio) needs this guarantee to safely poll futures.

```rust
// Creating a pinned boxed future:
Box::pin(async { "hello".to_string() })
// Returns: Pin<Box<dyn Future<Output = String>>>
```

**Swift doesn't have Pin** because Swift's ARC and heap allocation mean objects don't move in memory (they're always behind a reference).

## How Async Is Used in This Project

### The `new` Constructor

```rust
// From crates/snapshot-testing/src/snapshotting.rs
pub fn new<Snap, Fut>(
    path_extension: Option<&str>,
    diffing: Diffing<F>,
    snapshot: Snap,
) -> Self
where
    Snap: Fn(&V) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = F> + Send + 'static,
{
    Snapshotting {
        path_extension: path_extension.map(String::from),
        diffing,
        snapshot: Arc::new(move |v| Box::pin(snapshot(v))),
        //                              ^^^^^^^
        //                              Box::pin wraps the future in Pin<Box<...>>
    }
}
```

The user provides a function that returns any `Future` (`Fut`). The constructor wraps it with `Box::pin` to produce the `Pin<Box<dyn Future>>` that `Snapshotting` stores.

### Calling the Snapshot Function

```rust
// From crates/snapshot-testing/src/assert.rs
let snapshot = (snapshotting.snapshot)(value).await;
```

That's it. Call the closure with `&value`, get a future, `.await` it. The `.await` drives the future to completion and gives you the `F` value.

### Sync Strategies

Most snapshot strategies are synchronous -- they don't need async. For example, converting a value to its Debug string is instant. For these cases, you'd use `std::future::ready`:

```rust
// A synchronous snapshot that wraps its result in an immediately-ready future
Snapshotting::new(
    Some("txt"),
    Diffing::lines(),
    |value: &MyType| std::future::ready(format!("{:?}", value)),
)
```

`std::future::ready(value)` creates a future that is immediately ready -- `.await` on it returns the value without any actual async work. This is how most text-based strategies work.

**Swift equivalent:**
```swift
Snapshotting(
    pathExtension: "txt",
    diffing: .lines,
    snapshot: { Async(value: String(describing: $0)) }  // immediately resolves
)
```

## Async in verify_snapshot

```rust
pub async fn verify_snapshot<V, F>(
    value: &V,
    snapshotting: &Snapshotting<V, F>,
    ...
) -> Result<(), SnapshotError>
```

The function is `async` because it awaits the snapshot future:

```rust
let snapshot = (snapshotting.snapshot)(value).await;
```

Even though most strategies are synchronous (using `ready()`), the function is async to support strategies that genuinely need async (like rendering a web page or waiting for a network response).

## Tokio: The Async Runtime

Rust's async is "bring your own runtime." The language provides `async/await` syntax and the `Future` trait, but you need a runtime to actually execute futures. This project uses `tokio`, the most popular Rust async runtime.

In tests, you mark async tests with `#[tokio::test]`:

```rust
#[tokio::test]
async fn test_snapshot() {
    let result = verify_snapshot(&value, &strategy, None, &dir, "test").await;
    assert!(result.is_ok());
}
```

Without `#[tokio::test]`, the standard `#[test]` attribute doesn't know how to run async functions.

**Swift comparison:**
```swift
// Swift: async tests just work (built-in runtime)
func testSnapshot() async throws {
    // await works automatically
}
```

Swift's runtime is built in. Rust makes you choose (tokio, async-std, smol, etc.) -- more control, more setup.

## Timeouts

The Swift library uses `XCTestExpectation` with a timeout. In Rust with tokio:

```rust
// Conceptual timeout handling (from architecture doc)
use tokio::time::{timeout, Duration};

let result = timeout(
    Duration::from_secs(5),
    (snapshotting.snapshot)(value),
).await;

match result {
    Ok(snapshot) => { /* snapshot completed in time */ }
    Err(_elapsed) => { /* timed out */ }
}
```

`tokio::time::timeout` wraps any future with a deadline. If the future doesn't complete in time, it returns `Err(Elapsed)`.

## Exercise

The exercises for this lesson are covered in the other examples. To see async in action, look at the `verify_snapshot` function in `crates/snapshot-testing/src/assert.rs` and trace how:

1. The snapshot closure is called: `(snapshotting.snapshot)(value)`
2. The returned future is awaited: `.await`
3. The result is used for comparison

## What's Next

In [Lesson 08: Testing](./08-testing.md), we'll cover how Rust's testing ecosystem works -- `#[test]`, `#[ignore]`, doc tests, the `insta` crate, and how integration tests are organized in the `tests/` directory.
