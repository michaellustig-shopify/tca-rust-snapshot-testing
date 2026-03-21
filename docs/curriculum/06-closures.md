# Lesson 06: Closures

## The Big Idea

Closures are anonymous functions that can capture values from their surrounding scope. Swift closures and Rust closures look similar but have a critical difference: Rust has THREE closure traits (`Fn`, `FnMut`, `FnOnce`) that describe how a closure uses its captured values. This system lets the compiler reason about ownership inside closures without any runtime overhead.

This project is built on closures. Every `Snapshotting` and every `Diffing` stores closures. Understanding Rust's closure system is essential to understanding this codebase.

## The Three Closure Traits

| Trait | Can Be Called | Captures By | Swift Analogy |
|-------|-------------|-------------|---------------|
| `FnOnce` | Once | Move (takes ownership of captures) | A closure you `consume` |
| `FnMut` | Many times | Mutable reference | A closure that mutates its captures |
| `Fn` | Many times | Shared reference | A normal `@escaping` closure |

Every closure implements `FnOnce`. If it can be called multiple times without mutating, it also implements `FnMut` and `Fn`. The hierarchy is:

```
Fn (most restrictive, most useful)
 |
FnMut
 |
FnOnce (least restrictive)
```

## WHY Snapshot Functions Are `Fn`, Not `FnOnce`

Look at the snapshot closure in `Snapshotting`:

```rust
// From crates/snapshot-testing/src/snapshotting.rs
pub snapshot: Arc<dyn Fn(&V) -> Pin<Box<dyn Future<Output = F> + Send>> + Send + Sync>,
```

This is `Fn`, not `FnOnce` or `FnMut`. WHY:

1. **Called multiple times.** You might snapshot the same value with the same strategy multiple times. `FnOnce` can only be called once.

2. **No mutation needed.** A snapshot function reads a value and produces a format. It doesn't need to modify anything. `FnMut` would allow mutation but doesn't require it -- `Fn` is the right constraint.

3. **Thread safety.** `Fn` closures can be called from multiple threads simultaneously (no mutable state). This matters because tests run in parallel.

**Swift comparison:**
```swift
var snapshot: (Value) -> Async<Format>
```

In Swift, all closures that capture are reference types with the same calling convention. There's no Fn/FnMut/FnOnce distinction. Swift closures that mutate captured `var`s just... do it (with potential thread-safety issues).

## WHY `Arc<dyn Fn>` Not `fn` Pointers

Rust has two kinds of "callable things":

```rust
// Function pointer: points to a known function, no captures
fn add_one(x: i32) -> i32 { x + 1 }
let f: fn(i32) -> i32 = add_one;

// Closure: can capture values from the environment
let offset = 10;
let f = |x: i32| x + offset;  // captures `offset`
```

Function pointers (`fn(...)`) are simple: they're just addresses in memory, like C function pointers. But they can't capture anything.

Our strategies NEED captures. Look at `pullback`:

```rust
// From crates/snapshot-testing/src/snapshotting.rs
pub fn pullback<NewV, Transform>(self, transform: Transform) -> Snapshotting<NewV, F>
where
    NewV: 'static,
    Transform: Fn(&NewV) -> V + Send + Sync + 'static,
{
    let snapshot = self.snapshot;  // captured!
    Snapshotting::<NewV, F> {
        path_extension: self.path_extension,
        diffing: self.diffing,
        snapshot: Arc::new(move |new_v| {   // `move` takes ownership of `snapshot` and `transform`
            let v = transform(new_v);        // uses captured `transform`
            snapshot(&v)                     // uses captured `snapshot`
        }),
    }
}
```

The new closure captures `snapshot` (the old strategy's snapshot function) and `transform` (the conversion function). A function pointer can't do this -- it has no way to carry extra data.

## The `move` Keyword

In the pullback code above, notice `move |new_v| { ... }`:

```rust
snapshot: Arc::new(move |new_v| {
    let v = transform(new_v);
    snapshot(&v)
}),
```

Without `move`, the closure would borrow `snapshot` and `transform` by reference. But the closure is stored in an `Arc` and will outlive the function that created it -- the borrowed references would be dangling.

`move` tells the compiler: "take ownership of the captured variables, don't borrow them." After the `move` closure is created, `snapshot` and `transform` are moved into the closure and the original variables are gone.

**Swift comparison:**
```swift
// Swift: closures always capture by reference (with ARC keeping things alive)
snapshot = { [snapshot, transform] newV in  // capture list (optional, closures capture automatically)
    let v = transform(newV)
    return snapshot(v)
}
```

In Swift, capture lists are optional hints. In Rust, `move` is required when the closure outlives the scope where it was created.

## Type Erasure With `dyn Fn`

Every Rust closure has a unique, anonymous type. You can't name it:

```rust
let f = |x: &str| x.len();
// f has type `{anonymous closure #47}` -- you can't write this type
```

To store closures in struct fields, you need to erase the concrete type. Two options:

```rust
// Option 1: Box<dyn Fn> -- owned, single-owner
let f: Box<dyn Fn(&str) -> usize> = Box::new(|x| x.len());

// Option 2: Arc<dyn Fn> -- owned, shared (cloneable)
let f: Arc<dyn Fn(&str) -> usize> = Arc::new(|x| x.len());
```

`dyn Fn(...)` means "any type that implements the `Fn` trait with this signature." This is **dynamic dispatch** -- the closure is called through a vtable (a pointer to the actual function code), similar to protocol witnesses in Swift.

We use `Arc<dyn Fn>` because:
- `Box<dyn Fn>` can't be cloned (and `Snapshotting` needs `Clone`)
- `Arc<dyn Fn>` is cloneable (cloning just bumps the reference count)

## Closures in Diffing

All three functions in `Diffing` are closures:

```rust
// From crates/snapshot-testing/src/diffing.rs
pub struct Diffing<V: Clone> {
    pub to_data:   Arc<dyn Fn(&V) -> Vec<u8> + Send + Sync>,
    pub from_data: Arc<dyn Fn(&[u8]) -> V + Send + Sync>,
    pub diff:      Arc<dyn Fn(&V, &V) -> Option<(String, Vec<DiffAttachment>)> + Send + Sync>,
}
```

And the constructor accepts generic closure types:

```rust
pub fn new<TD, FD, D>(to_data: TD, from_data: FD, diff: D) -> Self
where
    TD: Fn(&V) -> Vec<u8> + Send + Sync + 'static,
    FD: Fn(&[u8]) -> V + Send + Sync + 'static,
    D: Fn(&V, &V) -> Option<(String, Vec<DiffAttachment>)> + Send + Sync + 'static,
{
    Diffing {
        to_data: Arc::new(to_data),
        from_data: Arc::new(from_data),
        diff: Arc::new(diff),
    }
}
```

The constructor takes concrete closure types (`TD`, `FD`, `D`) -- the compiler knows the exact types and can optimize. Then it wraps them in `Arc<dyn Fn>` for storage. This is the best of both worlds: zero-cost abstraction at the call site, dynamic dispatch for storage.

## The DiffTool Closure

`DiffTool` in `config.rs` follows the same pattern:

```rust
// From crates/snapshot-testing/src/config.rs
pub struct DiffTool {
    command: Arc<dyn Fn(&str, &str) -> String + Send + Sync>,
}

impl DiffTool {
    pub fn new<F>(command: F) -> Self
    where
        F: Fn(&str, &str) -> String + Send + Sync + 'static,
    {
        DiffTool {
            command: Arc::new(command),
        }
    }
}
```

Usage from the test helpers:

```rust
// From crates/snapshot-testing/tests/helpers.rs
DiffTool::new(|a, b| format!("ksdiff {} {}", a, b))
```

The closure `|a, b| format!(...)` captures nothing (it only uses its parameters), so it could technically be a function pointer. But `Arc<dyn Fn>` is used for consistency with the rest of the API.

## `with_snapshot_testing` and `FnOnce`

Not every function uses `Fn`. Look at `with_snapshot_testing`:

```rust
// From crates/snapshot-testing/src/config.rs
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

Here `f` is `FnOnce` -- it's only called once. This is the right choice because:

1. The closure runs exactly once (between push and pop)
2. `FnOnce` is the most permissive -- it accepts any closure, even ones that move captures
3. Swift's equivalent takes a non-escaping closure, which is similar (called exactly once, synchronously)

**Swift comparison:**
```swift
func withSnapshotTesting<R>(
    record: Record? = nil,
    operation: () throws -> R
) rethrows -> R
```

Swift's `operation` is non-escaping by default (no `@escaping`), meaning it's called during the function and doesn't outlive it. Rust's `FnOnce` expresses a similar guarantee.

## Exercise

Run the exercise for this lesson:

```bash
cargo run -p snapshot-testing --example ex03_pullback
```

Open `crates/snapshot-testing/examples/ex03_pullback.rs`. Focus on:

- How closures capture variables from pullback
- The `move` keyword in closure creation
- Chaining pullbacks (each creates a new closure that captures the previous one)

## What's Next

In [Lesson 07: Async Rust](./07-async.md), we'll look at the `Pin<Box<dyn Future>>` type in the snapshot closure, how Rust's async/await differs from Swift's, and why we use pinned boxed futures for type erasure.
