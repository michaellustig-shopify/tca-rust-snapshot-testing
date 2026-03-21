# Lesson 02: Lifetimes

## The Big Idea

In [Lesson 01](./01-ownership-basics.md), we saw that references let you borrow data without owning it. But there's a catch: every reference has a **lifetime** -- a scope during which the reference is valid. Rust's compiler tracks lifetimes to guarantee you never use a reference after the data it points to has been freed.

Most of the time, lifetimes are invisible -- the compiler figures them out automatically ("lifetime elision"). But when you store references inside structs, lifetimes become explicit and contagious. This lesson explains why we chose `Arc` over references for the closures in `Snapshotting` and `Diffing`.

## Lifetimes: The 30-Second Version

A lifetime is written `'a` (apostrophe + name). It tells the compiler: "this reference is valid for at least this long."

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

This says: "I take two string slices that live at least as long as `'a`, and I return a string slice that also lives at least as long as `'a`." The compiler uses this to verify that the returned reference is valid wherever the caller uses it.

**Swift doesn't have this.** Swift uses ARC (automatic reference counting) to keep data alive as long as any reference exists. Rust doesn't have ARC built into the language -- instead, the compiler proves statically that references are valid. No runtime cost.

## What Would Happen If We Used References in Diffing

Here's the actual `Diffing` struct:

```rust
// crates/snapshot-testing/src/diffing.rs (what we actually have)
pub struct Diffing<V: Clone> {
    pub to_data:   Arc<dyn Fn(&V) -> Vec<u8> + Send + Sync>,
    pub from_data: Arc<dyn Fn(&[u8]) -> V + Send + Sync>,
    pub diff:      Arc<dyn Fn(&V, &V) -> Option<(String, Vec<DiffAttachment>)> + Send + Sync>,
}
```

Now imagine we tried to use references instead of `Arc`:

```rust
// HYPOTHETICAL: What if Diffing stored references to closures?
pub struct Diffing<'a, V: Clone> {
    pub to_data:   &'a dyn Fn(&V) -> Vec<u8>,
    pub from_data: &'a dyn Fn(&[u8]) -> V,
    pub diff:      &'a dyn Fn(&V, &V) -> Option<(String, Vec<DiffAttachment>)>,
}
```

That `'a` is a lifetime parameter. It says: "this Diffing is only valid as long as the closures it references are alive." This sounds fine until you see the cascade.

## The Lifetime Explosion

`Snapshotting` contains a `Diffing`:

```rust
// HYPOTHETICAL: Snapshotting with lifetime-annotated Diffing
pub struct Snapshotting<'a, V, F: Clone> {
    pub path_extension: Option<String>,
    pub diffing: Diffing<'a, F>,   // <-- inherits 'a
    pub snapshot: &'a dyn Fn(&V) -> Pin<Box<dyn Future<Output = F> + Send>>,  // also 'a
}
```

Now `Snapshotting` has a lifetime parameter too. That means every function that accepts or returns a `Snapshotting` must carry the lifetime:

```rust
// HYPOTHETICAL: verify_snapshot with lifetime
pub async fn verify_snapshot<'a, V, F>(
    value: &V,
    snapshotting: &Snapshotting<'a, V, F>,
    ...
) -> Result<(), SnapshotError>
```

And `pullback` gets even worse:

```rust
// HYPOTHETICAL: pullback with lifetimes
impl<'a, V, F: Clone> Snapshotting<'a, V, F> {
    pub fn pullback<'b, NewV>(
        &'a self,
        transform: &'b dyn Fn(&NewV) -> V,
    ) -> Snapshotting<???, NewV, F>
    //                ^^^ What lifetime goes here?
    //                    The new strategy borrows both the old strategy AND the transform.
    //                    Now you need to express that the returned Snapshotting
    //                    can't outlive EITHER 'a or 'b.
}
```

This is the "lifetime explosion." Every time you compose strategies (and the whole point of this library is composability), you add another lifetime that must be threaded through every type and function.

## WHY We Chose Arc Instead

`Arc` (Atomic Reference Count) is Rust's explicit equivalent of Swift's ARC. It wraps a value in a reference-counted pointer. When you clone an `Arc`, it increments the count. When the last `Arc` drops, the value is freed.

```rust
// What we actually have:
pub struct Diffing<V: Clone> {
    pub to_data:   Arc<dyn Fn(&V) -> Vec<u8> + Send + Sync>,
    pub from_data: Arc<dyn Fn(&[u8]) -> V + Send + Sync>,
    pub diff:      Arc<dyn Fn(&V, &V) -> Option<(String, Vec<DiffAttachment>)> + Send + Sync>,
}
```

No lifetime parameter. `Diffing<V>` owns its closures via `Arc`. You can clone it, store it, pass it around, return it from functions -- no lifetime tracking needed. The closures live as long as anyone holds an `Arc` to them.

**The tradeoff:**

| Approach | Pros | Cons |
|----------|------|------|
| `&'a dyn Fn` | Zero runtime overhead | Lifetime explosion, hard to compose, can't Clone |
| `Box<dyn Fn>` | No lifetimes, owned | Can't Clone (Box is single-owner) |
| `Arc<dyn Fn>` | No lifetimes, Clone-able | Small runtime cost (ref counting) |

We chose `Arc` because:

1. **`Snapshotting` must be Clone-able.** Strategies get shared -- you create one and use it in multiple tests, or chain it through multiple pullbacks. `Box<dyn Fn>` can't be cloned. `Arc<dyn Fn>` can.

2. **The runtime cost is negligible.** We're in test code, not a hot loop. An atomic increment/decrement per clone is invisible.

3. **The API stays clean.** No lifetime parameters anywhere in the public API. Users never write `Snapshotting<'a, V, F>` -- just `Snapshotting<V, F>`.

## The `'static` Bound

You'll see `'static` on our type bounds:

```rust
impl<V, F> Snapshotting<V, F>
where
    F: Clone + Send + 'static,
    V: 'static,
```

`'static` doesn't mean "lives forever." It means "doesn't contain any non-static references." In other words: the type is self-contained, with no borrowed data inside it.

This is required because our closures are stored in `Arc`, and `Arc` contents must be `'static` -- they could live for any duration (since `Arc` controls their lifetime), so they can't hold references that might expire.

**In practice:** This is almost never a limitation. Structs, enums, `String`, `Vec<u8>`, numbers -- all are `'static`. The only things that aren't `'static` are types that contain borrowed references like `&'a str`. Since snapshot values are typically data you construct in tests, they're almost always `'static`.

## Swift Comparison: Why Swift Doesn't Have This Problem

```swift
// Swift: closures are reference-counted automatically
struct Diffing<Value> {
    var toData: (Value) -> Data
    var fromData: (Data) -> Value
    var diff: (Value, Value) -> (String, [XCTAttachment])?
}
```

No lifetime annotations, no `Arc`. Swift closures are reference types with automatic reference counting (ARC). When you store a closure in a struct, Swift automatically manages the closure's lifetime. When no one references it anymore, it gets freed.

Rust gives you the same behavior with `Arc`, but you have to opt in. The benefit: Rust makes the cost visible. You know exactly where reference counting happens and where it doesn't. In performance-critical code (not test code), this matters.

## When You DO See Lifetimes in This Project

Lifetimes appear in function parameters, where the compiler often infers them:

```rust
// From crates/snapshot-testing/src/assert.rs
pub async fn verify_snapshot<V, F>(
    value: &V,              // lifetime elided -- compiler infers it
    snapshotting: &Snapshotting<V, F>,
    name: Option<&str>,     // different lifetime, also elided
    snapshot_dir: &Path,
    test_name: &str,
)
```

Each `&` has an implicit lifetime. The compiler knows that `value` must live at least as long as the function call. It figures this out without you writing `<'a>`.

You'd only need explicit lifetimes if the return type contained a reference whose lifetime depends on the inputs. Since `verify_snapshot` returns `Result<(), SnapshotError>` (all owned data), no explicit lifetimes are needed.

## Lifetime Elision Rules

Rust has three rules for automatically figuring out lifetimes, so you don't have to write them manually most of the time:

1. **Each reference parameter gets its own lifetime.** `fn foo(a: &str, b: &str)` becomes `fn foo<'a, 'b>(a: &'a str, b: &'b str)`.

2. **If there's exactly one input lifetime, the output gets that lifetime.** `fn foo(s: &str) -> &str` becomes `fn foo<'a>(s: &'a str) -> &'a str`.

3. **If one of the inputs is `&self` or `&mut self`, the output gets `self`'s lifetime.** This is the method rule.

These rules cover most cases. When they don't (e.g., two input lifetimes and a reference output), you must write lifetimes explicitly:

```rust
// Compiler can't figure out which input lifetime the output should use
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

In this project, we rarely need explicit lifetimes because:
- Functions return owned types (`String`, `Vec<u8>`, `Result<(), SnapshotError>`)
- Closures are stored in `Arc` (owned, no lifetime needed)
- References in parameters have elided lifetimes

## The Cost Comparison

| Approach | Compile-time cost | Runtime cost | API complexity |
|----------|------------------|-------------|----------------|
| References with lifetimes | Zero runtime cost | Zero | High (lifetime params everywhere) |
| Arc (what we chose) | Slightly longer compile | Atomic ref count ops | Low (no lifetimes in API) |
| Clone everything | Zero API complexity | Copies all data | Low but wasteful |

For test infrastructure like this library, Arc's runtime cost is negligible. The API simplicity is worth far more than saving a few atomic operations.

## Exercise

Run the exercise for this lesson:

```bash
cargo run -p snapshot-testing --example ex02_custom_strategy
```

Open `crates/snapshot-testing/examples/ex02_custom_strategy.rs`. It walks you through:

- Creating a `Diffing<String>` with Arc-wrapped closures
- Creating a `Snapshotting<MyType, String>`
- Seeing how `Arc` makes these types cloneable
- A commented-out experiment to try adding lifetime parameters and see the compiler errors

## What's Next

In [Lesson 03: Traits](./03-traits.md), we'll explore why `Snapshotting` is a struct with closures rather than a trait you implement. This is the core architectural decision of the library -- it determines everything about how strategies compose.
