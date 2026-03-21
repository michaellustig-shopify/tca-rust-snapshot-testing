# Lesson 03: Traits

## The Big Idea

Swift protocols and Rust traits serve the same purpose: they define a contract that types can conform to. But in this project, we made a deliberate choice NOT to use a trait for the core abstraction. `Snapshotting` is a struct, not a trait. Understanding why requires understanding what traits can and can't do in Rust.

## Swift Protocols vs Rust Traits: Quick Comparison

```swift
// Swift protocol
protocol Snapshotable {
    associatedtype Format
    func snapshot() -> Format
}
```

```rust
// Rust trait (hypothetical)
trait Snapshotable {
    type Format;
    fn snapshot(&self) -> Self::Format;
}
```

The syntax is similar, but there's a crucial difference in how they're used for our problem.

## The Trait-Based Approach (What We Did NOT Do)

Imagine if we defined a trait for snapshot-testable types:

```rust
// HYPOTHETICAL: Trait-based approach
trait Snapshotable {
    type Format: Clone;
    fn snapshot(&self) -> Self::Format;
    fn diff(old: &Self::Format, new: &Self::Format) -> Option<String>;
}

// Usage:
impl Snapshotable for User {
    type Format = String;
    fn snapshot(&self) -> String {
        format!("{:?}", self)
    }
    fn diff(old: &String, new: &String) -> Option<String> {
        line_diff(old, new, 3)
    }
}
```

This works for simple cases. But it has a fundamental flaw.

## WHY We Chose Structs Over Traits

**Problem 1: One trait implementation per type.**

With traits, `User` can only implement `Snapshotable` once. You pick ONE snapshot format -- maybe Debug output. But what if you also want JSON? And a custom pretty-print? In Swift, you'd use different `Snapshotting` values:

```swift
// Swift: Multiple strategies for the same type
assertSnapshot(of: user, as: .dump)       // Debug output
assertSnapshot(of: user, as: .json)       // JSON
assertSnapshot(of: user, as: .description) // Display string
```

With a trait, you'd need newtype wrappers:

```rust
// HYPOTHETICAL: Clunky newtype wrappers to get multiple formats
struct AsDebug<T>(T);
struct AsJson<T>(T);
struct AsDescription<T>(T);

impl<T: Debug> Snapshotable for AsDebug<T> { ... }
impl<T: Serialize> Snapshotable for AsJson<T> { ... }
impl<T: Display> Snapshotable for AsDescription<T> { ... }

// Usage becomes awkward:
assert_snapshot(&AsDebug(&user));
assert_snapshot(&AsJson(&user));
```

**Problem 2: Traits can't compose via pullback.**

The `pullback` operation is the core of this library's composability. It transforms a strategy for type A into a strategy for type B, given a function `B -> A`. With structs:

```rust
// Real code from crates/snapshot-testing/src/snapshotting.rs
pub fn pullback<NewV, Transform>(self, transform: Transform) -> Snapshotting<NewV, F>
where
    NewV: 'static,
    Transform: Fn(&NewV) -> V + Send + Sync + 'static,
{
    let snapshot = self.snapshot;
    Snapshotting::<NewV, F> {
        path_extension: self.path_extension,
        diffing: self.diffing,
        snapshot: Arc::new(move |new_v| {
            let v = transform(new_v);
            snapshot(&v)
        }),
    }
}
```

This creates a new `Snapshotting` value by wrapping the old one's snapshot closure. It's just data transformation -- no type system gymnastics.

With traits, pullback would require generic associated types, higher-kinded types, or other advanced features that are either complex or not yet fully available in Rust.

**Problem 3: Strategies for types you don't own.**

Traits can only be implemented by the crate that defines the type OR the crate that defines the trait (the "orphan rule"). If you want to snapshot `serde_json::Value`, you can't implement a trait for it in your test file.

With struct-based strategies, no problem:

```rust
// Anyone can create a strategy for any type
let json_strategy: Snapshotting<serde_json::Value, String> = Snapshotting::new(
    Some("json"),
    Diffing::lines(),
    |v: &serde_json::Value| async { serde_json::to_string_pretty(v).unwrap() },
);
```

## The Struct-Based Pattern

Here's the actual `Snapshotting` struct:

```rust
// From crates/snapshot-testing/src/snapshotting.rs
#[derive(Clone)]
pub struct Snapshotting<V, F>
where
    F: Clone,
{
    pub path_extension: Option<String>,
    pub diffing: Diffing<F>,
    pub snapshot: Arc<dyn Fn(&V) -> Pin<Box<dyn Future<Output = F> + Send>> + Send + Sync>,
}
```

It's a **value that carries behavior**. The behavior (how to snapshot, how to diff) lives in closures, not in trait implementations. This is the same pattern as Swift's original library.

**Swift equivalent:**
```swift
struct Snapshotting<Value, Format> {
    var pathExtension: String?
    var diffing: Diffing<Format>
    var snapshot: (Value) -> Async<Format>
}
```

Nearly identical. The Rust version just makes the closure types explicit.

## Traits We DO Use: Derive Macros

While `Snapshotting` itself isn't a trait, the project uses traits extensively for standard behavior. Look at the `Record` enum:

```rust
// From crates/snapshot-testing/src/config.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Record {
    All,
    Failed,
    Missing,
    Never,
}
```

The `#[derive(...)]` attribute automatically generates trait implementations:

| Trait | What It Gives You | Swift Equivalent |
|-------|-------------------|------------------|
| `Debug` | `{:?}` formatting in println/format | `CustomDebugStringConvertible` |
| `Clone` | `.clone()` method to copy the value | Value type semantics (automatic in Swift) |
| `Copy` | Implicit copy on assignment (no `.clone()` needed) | Like Swift value types |
| `PartialEq` | `==` operator | `Equatable` |
| `Eq` | Marker that equality is reflexive (a == a is always true) | *(implicit in Swift's Equatable)* |

`Copy` deserves special attention. In Rust, assignment normally moves:

```rust
let a = Record::All;
let b = a;     // `a` is moved to `b`
// println!("{:?}", a);  // ERROR: a was moved
```

But with `Copy`, assignment copies instead:

```rust
let a = Record::All;
let b = a;     // `a` is copied to `b`
println!("{:?}", a);  // Fine! `a` is still valid
```

`Copy` only works for types that are cheap to copy (small, no heap allocation). `Record` is an enum with no data -- just 4 variants -- so `Copy` is appropriate. `Snapshotting` can't be `Copy` because it contains `Arc` and `String` (heap-allocated).

## Traits Used by Built-in Strategies

The architecture doc describes which standard traits enable which strategies:

```
std::fmt::Debug   --> enables Snapshotting::debug()
std::fmt::Display --> enables Snapshotting::description()
serde::Serialize  --> enables Snapshotting::json()
```

These are standard Rust traits. Any type that derives `Debug` can be snapshot-tested with the debug strategy. Any type that implements `Serialize` (from the `serde` crate) can use the JSON strategy. But the type doesn't need to know about snapshot testing at all -- the strategy is external.

**Swift comparison:**
```swift
// Swift: Same pattern -- Debug and Encodable are standard protocols
assertSnapshot(of: user, as: .dump)   // requires CustomDumpStringConvertible
assertSnapshot(of: user, as: .json)   // requires Encodable
```

## Custom Debug Implementation

`Snapshotting` can't derive `Debug` because it contains `Arc<dyn Fn>` (closures aren't debuggable). So it implements `Debug` manually:

```rust
// From crates/snapshot-testing/src/snapshotting.rs
impl<V, F> std::fmt::Debug for Snapshotting<V, F>
where
    F: Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Snapshotting")
            .field("path_extension", &self.path_extension)
            .finish_non_exhaustive()
    }
}
```

`finish_non_exhaustive()` prints `..` to indicate there are fields not shown. The output looks like:

```
Snapshotting { path_extension: Some("txt"), .. }
```

**Swift equivalent:**
```swift
extension Snapshotting: CustomDebugStringConvertible {
    var debugDescription: String {
        "Snapshotting(pathExtension: \(pathExtension ?? "nil"), ...)"
    }
}
```

## The `DiffAttachment` Enum and Trait Derives

```rust
// From crates/snapshot-testing/src/diffing.rs
#[derive(Debug, Clone)]
pub enum DiffAttachment {
    Data { bytes: Vec<u8>, name: String },
}
```

This is a Rust enum with named fields (like a Swift enum with associated values):

```swift
// Swift equivalent
enum DiffAttachment {
    case data(Data, name: String)
}
```

`Debug` and `Clone` are derived because `Vec<u8>` and `String` both implement those traits. Rust's derive macros only work if all fields implement the required trait.

## Exercise

Run the exercise for this lesson:

```bash
cargo run -p snapshot-testing --example ex02_custom_strategy
```

This is the same exercise as Lesson 02, but now focus on:

- How `Snapshotting` is constructed as a value (not implementing a trait)
- Creating multiple strategies for the same type
- Using `Debug` derive on your custom types

## What's Next

In [Lesson 04: Generics](./04-generics.md), we'll look at the type parameters `V` and `F` in `Snapshotting<V, F>`, how Rust's generics compare to Swift's, and what those `where` clauses with trait bounds mean.
