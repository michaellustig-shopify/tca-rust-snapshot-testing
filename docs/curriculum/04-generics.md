# Lesson 04: Generics and Trait Bounds

## The Big Idea

Swift and Rust both have generics -- type parameters that let you write code that works with many types. But Rust generics have a key difference: **monomorphization**. When you write `Snapshotting<User, String>`, the Rust compiler generates a specialized version of every method for that exact combination of types. Swift does something similar for value types but uses dynamic dispatch for protocols.

This lesson explains the generic parameters in this project, what trait bounds mean, and how Rust's approach differs from Swift's.

## The Generic Parameters: V and F

```rust
// From crates/snapshot-testing/src/snapshotting.rs
pub struct Snapshotting<V, F>
where
    F: Clone,
{
    pub path_extension: Option<String>,
    pub diffing: Diffing<F>,
    pub snapshot: Arc<dyn Fn(&V) -> Pin<Box<dyn Future<Output = F> + Send>> + Send + Sync>,
}
```

Two type parameters:

| Parameter | Name | What It Represents | Examples |
|-----------|------|--------------------|----------|
| `V` | Value | The thing you're snapshotting | `User`, `HttpRequest`, `Vec<String>` |
| `F` | Format | The serialized form for diffing | `String`, `Vec<u8>` |

A `Snapshotting<User, String>` knows how to take a `User` and produce a `String` for comparison. A `Snapshotting<HttpRequest, String>` does the same for HTTP requests.

**Swift comparison:**
```swift
// Swift uses the exact same two-parameter design
struct Snapshotting<Value, Format> { ... }

// Type alias for when value IS the format:
typealias SimplySnapshotting<Format> = Snapshotting<Format, Format>
```

Rust has the same type alias:

```rust
type SimplySnapshotting<F> = Snapshotting<F, F>;
```

## Trait Bounds: The `where` Clause

Look at the `impl` block:

```rust
impl<V, F> Snapshotting<V, F>
where
    F: Clone + Send + 'static,
    V: 'static,
{
    pub fn new<Snap, Fut>(...) -> Self
    where
        Snap: Fn(&V) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = F> + Send + 'static,
    { ... }
}
```

The `where` clause lists **trait bounds** -- requirements that type parameters must satisfy. Let's break each one down:

### `F: Clone`

The format type must be cloneable. This appears on the struct definition itself:

```rust
pub struct Snapshotting<V, F>
where
    F: Clone,
```

WHY: `F` is stored inside `Diffing<F>`, which needs to clone format values during comparison. For example, when loading a snapshot from disk, `from_data` produces an `F` value that gets passed to `diff`. Both operations might need their own copy.

### `F: Send`

The format type must be safe to send between threads.

WHY: The snapshot future returns `F` (via `Pin<Box<dyn Future<Output = F> + Send>>`). Since tests run on different threads and async runtimes may move futures between threads, `F` must be `Send`. We'll cover `Send` more in [Lesson 10](./10-advanced.md).

### `V: 'static` and `F: 'static`

The value and format types must not contain borrowed references. (See [Lesson 02](./02-lifetimes.md) for why `'static` doesn't mean "lives forever.")

WHY: The snapshot closure is stored in an `Arc`, which requires its contents to be `'static`. If `V` contained a `&'a str`, the closure `Fn(&V) -> ...` would inherit that lifetime, and it couldn't be wrapped in `Arc`.

### `Snap: Fn(&V) -> Fut + Send + Sync + 'static`

The snapshot function must:
- Be callable with a `&V` (`Fn(&V) -> Fut`)
- Be sendable between threads (`Send`)
- Be callable from any thread simultaneously (`Sync`)
- Not contain borrowed references (`'static`)

WHY: The closure is stored in `Arc<dyn Fn(...) + Send + Sync>`. `Arc` can be shared across threads, so the closure must be safe to call from any thread (`Sync`) and to move between threads (`Send`).

## Swift Generics vs Rust Generics

### Monomorphization

When you write:

```rust
let strategy: Snapshotting<User, String> = ...;
```

The compiler generates a concrete version of `Snapshotting` specialized for `User` and `String`. All method calls on this type are resolved at compile time with zero overhead. This is called **monomorphization**.

Swift does something similar for structs and enums (value types with generics), but uses "witness tables" (similar to vtables) for protocol-typed values.

### Where the Analogy Breaks Down

In Swift, you can write:

```swift
func snapshot<T: Encodable>(value: T) -> String {
    let data = try! JSONEncoder().encode(value)
    return String(data: data, encoding: .utf8)!
}
```

The `Encodable` constraint is enough. In Rust, the equivalent uses `serde::Serialize`:

```rust
fn snapshot<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).unwrap()
}
```

The syntax is different but the concept is the same: "T can be any type, as long as it implements this trait/protocol."

### Multiple Bounds

Rust uses `+` to combine bounds:

```rust
F: Clone + Send + 'static
```

Swift uses `&` in protocol composition or `where` clauses:

```swift
func doSomething<T: Codable & Sendable>(value: T) { ... }
// or
func doSomething<T>(value: T) where T: Codable, T: Sendable { ... }
```

## Generic Methods: `pullback`

The `pullback` method introduces a NEW type parameter:

```rust
pub fn pullback<NewV, Transform>(self, transform: Transform) -> Snapshotting<NewV, F>
where
    NewV: 'static,
    Transform: Fn(&NewV) -> V + Send + Sync + 'static,
```

This says: "Given a `Snapshotting<V, F>`, and a function that converts `&NewV -> V`, produce a `Snapshotting<NewV, F>`."

**Swift equivalent:**
```swift
func pullback<NewValue>(_ transform: @escaping (NewValue) -> Value)
    -> Snapshotting<NewValue, Format>
```

The key difference: Rust's `Transform` is a generic parameter for the closure type. This means the compiler knows the exact closure type at compile time and can inline it. Swift's `@escaping (NewValue) -> Value` is type-erased -- it always goes through dynamic dispatch.

In our case, we type-erase the closure too (wrapping it in `Arc<dyn Fn>`), so the performance is similar to Swift. But Rust gives you the choice.

## The Diffing Generic

`Diffing` has just one type parameter:

```rust
// From crates/snapshot-testing/src/diffing.rs
pub struct Diffing<V>
where
    V: Clone,
{
    pub to_data:   Arc<dyn Fn(&V) -> Vec<u8> + Send + Sync>,
    pub from_data: Arc<dyn Fn(&[u8]) -> V + Send + Sync>,
    pub diff:      Arc<dyn Fn(&V, &V) -> Option<(String, Vec<DiffAttachment>)> + Send + Sync>,
}
```

`V` here is the format type (the same as `F` in `Snapshotting`). `Diffing<String>` knows how to serialize, deserialize, and compare strings. `Diffing<Vec<u8>>` does the same for byte arrays.

The `Clone` bound on `V` is needed because `Diffing` might need to produce owned copies of format values (e.g., `from_data` returns an owned `V`).

## Type Aliases for Readability

When `V == F` (the value IS the format), the type signature simplifies:

```rust
type SimplySnapshotting<F> = Snapshotting<F, F>;
```

A `SimplySnapshotting<String>` takes a `String` and produces a `String` -- the identity transformation. `Diffing::lines()` would return a `Diffing<String>`, and `SimplySnapshotting::lines()` would combine that with an identity snapshot function.

## Exercise

Run the exercise for this lesson:

```bash
cargo run -p snapshot-testing --example ex03_pullback
```

Open `crates/snapshot-testing/examples/ex03_pullback.rs`. It walks you through:

- Creating a generic function with trait bounds
- Using `pullback` to transform a `Snapshotting<String, String>` into a `Snapshotting<i32, String>`
- Chaining multiple pullbacks to build complex strategies from simple ones

## What's Next

In [Lesson 05: Error Handling](./05-error-handling.md), we'll look at `Result<T, E>`, the `thiserror` derive macro, and why `verify_snapshot` returns a `Result` while `assert_snapshot` panics.
