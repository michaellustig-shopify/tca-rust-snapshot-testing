# Lesson 01: Ownership and Borrowing

## The Big Idea

In Swift, you can pass values around without thinking much about who "owns" them. Value types get copied, reference types are reference-counted automatically (ARC). You never write code to manage memory.

Rust is different. Every value has exactly one owner. When ownership transfers (a "move"), the old variable becomes invalid. To let someone use a value without taking ownership, you lend them a reference (a "borrow"). The compiler enforces all of this at compile time -- no runtime cost, no garbage collector, no reference counting.

## Ownership In This Project: `verify_snapshot`

Open `crates/snapshot-testing/src/assert.rs` and look at this signature:

```rust
pub async fn verify_snapshot<V, F>(
    value: &V,                           // borrowed
    snapshotting: &Snapshotting<V, F>,   // borrowed
    name: Option<&str>,                  // borrowed
    snapshot_dir: &Path,                 // borrowed
    test_name: &str,                     // borrowed
) -> Result<(), SnapshotError>
```

Every parameter is borrowed (the `&` prefix). None of them are owned. Compare to the Swift equivalent:

```swift
func verifySnapshot<Value, Format>(
    of value: Value,                          // passed by value (but may be reference type)
    as snapshotting: Snapshotting<Value, Format>,  // passed by value
    named name: String?,
    ...
) -> String?
```

### Swift vs Rust: What Happens When You Pass a Value

**Swift:**
```swift
let user = User(name: "Blobby")
assertSnapshot(of: user, as: .dump)
print(user.name)  // Fine -- `user` is still valid
```

Swift copies value types or increments the reference count for reference types. Either way, you can keep using `user` after the call.

**Rust without borrowing (hypothetical):**
```rust
let user = User { name: "Blobby".into() };
assert_snapshot(user, &Snapshotting::dump());  // Takes ownership of `user`
println!("{}", user.name);  // COMPILE ERROR: `user` was moved
```

If `assert_snapshot` took `user` by value, it would *consume* the value. After the call, `user` is gone. You couldn't use it again, print it, or pass it to another function.

**Rust with borrowing (what we actually do):**
```rust
let user = User { name: "Blobby".into() };
verify_snapshot(&user, &Snapshotting::dump(), None, dir, "test").await;
println!("{}", user.name);  // Fine -- we only lent a reference
```

The `&` means "borrow, don't take." The function can read `user` but can't consume or modify it. When the function returns, the borrow ends and `user` is still yours.

## WHY `verify_snapshot` Takes `&V`, Not `V`

Three reasons:

1. **You want to keep using the value.** In tests, you often snapshot a value and then make more assertions about it. Taking ownership would force you to clone it first.

2. **No `Clone` requirement.** If we took `V` by value, every type you snapshot would need to implement `Clone` (so you could copy it before passing). With `&V`, any type works -- even types that can't be cloned (like file handles or network connections).

3. **Efficiency.** For large structs, passing by reference avoids copying all the data. A reference is just a pointer (8 bytes on 64-bit).

## WHY `Snapshotting.snapshot` Takes `&V`

Look at the snapshot closure type in `crates/snapshot-testing/src/snapshotting.rs`:

```rust
pub snapshot: Arc<dyn Fn(&V) -> Pin<Box<dyn Future<Output = F> + Send>> + Send + Sync>,
```

The closure takes `&V` -- a reference to the value. This is important for the same reason: the snapshot function shouldn't consume the value. It reads it, transforms it into the format type `F`, and returns the result.

If it took `V` by value:
```rust
// BAD: Consumes value on every call
snapshot: Arc<dyn Fn(V) -> Pin<Box<dyn Future<Output = F> + Send>>>
```

Then calling the snapshot function would destroy the value. You couldn't snapshot the same value twice (which you might want to do with different strategies).

## The Three Kinds of References

Rust has three ways to pass values:

| Syntax | Name | Can Read? | Can Modify? | Moves? | Swift Analogy |
|--------|------|-----------|-------------|--------|---------------|
| `v` | Owned (move) | Yes | Yes | Yes | Consuming a value type |
| `&v` | Shared reference | Yes | No | No | `let` parameter |
| `&mut v` | Mutable reference | Yes | Yes | No | `inout` parameter |

In this project, we almost always use shared references (`&`). The Diffing struct's functions show all three patterns:

```rust
// From crates/snapshot-testing/src/diffing.rs

// to_data: borrows the value (reads it to produce bytes)
pub to_data: Arc<dyn Fn(&V) -> Vec<u8> + Send + Sync>,

// from_data: borrows bytes (reads them to produce a value)
pub from_data: Arc<dyn Fn(&[u8]) -> V + Send + Sync>,

// diff: borrows both values (reads them to compare)
pub diff: Arc<dyn Fn(&V, &V) -> Option<(String, Vec<DiffAttachment>)> + Send + Sync>,
```

Every function borrows its inputs. None of them need to consume or modify the data -- they just read it and produce new owned values as output.

## `Option<&str>` vs `Option<String>`

Look at the `name` parameter in `verify_snapshot`:

```rust
name: Option<&str>,  // borrowed string slice
```

Not `Option<String>`. In Rust, `String` is an owned, heap-allocated string. `&str` is a borrowed slice (a pointer + length, no allocation). Since `verify_snapshot` only needs to read the name (to build a file path), it borrows a slice.

**Swift equivalent:**
```swift
named name: String?  // Swift strings are copy-on-write value types
```

Swift doesn't distinguish between owned and borrowed strings. `String` is always a value type with copy-on-write semantics. Rust makes the distinction explicit: if you own a `String`, you can modify it and you're responsible for freeing it. If you have a `&str`, you're just reading someone else's string.

## The Borrowing Rules

Rust enforces two rules at compile time:

1. **At any time, you can have EITHER** one mutable reference (`&mut`) **OR** any number of shared references (`&`). Never both.

2. **References must not outlive the data they point to.** (This is what lifetimes enforce -- see [Lesson 02](./02-lifetimes.md).)

These rules prevent data races and use-after-free bugs at compile time. No runtime checks needed.

## Practical Pattern: Borrow In, Own Out

A common pattern in this codebase is: borrow input, return owned output. Look at `line_diff`:

```rust
// From crates/snapshot-testing/src/diff.rs
pub fn line_diff(old: &str, new: &str, context_lines: usize) -> Option<String> {
    //           ^^^^^       ^^^^^   borrows input
    //                                                         ^^^^^^ returns owned output
    if old == new {
        return None;
    }
    let diff = TextDiff::from_lines(old, new);
    let mut output = String::new();   // new owned String
    for hunk in diff.unified_diff().context_radius(context_lines).iter_hunks() {
        output.push_str(&format!("{hunk}"));
    }
    // ...
    Some(output)  // returns the owned String
}
```

The function borrows `old` and `new` (reads them), creates a brand new `String` containing the diff, and returns it. The caller owns the result. This avoids copying the input strings while giving the caller full ownership of the output.

**Swift equivalent:** In Swift, the function would look nearly identical, but you wouldn't think about ownership:

```swift
func lineDiff(old: String, new: String, contextLines: Int) -> String? {
    // Swift copies `old` and `new` if needed (copy-on-write)
    // Caller owns the returned String? automatically
}
```

## Ownership in Config: `with_snapshot_testing`

Another example from the codebase. Look at `with_snapshot_testing` in `crates/snapshot-testing/src/config.rs`:

```rust
pub fn with_snapshot_testing<R, F: FnOnce() -> R>(
    config: SnapshotTestingConfiguration,  // takes ownership!
    f: F,
) -> R
```

Here, `config` is taken by value (owned), not borrowed. WHY? Because the function pushes it onto a thread-local stack:

```rust
CONFIG_STACK.with(|stack| {
    stack.borrow_mut().push(config);  // config is moved into the stack
});
```

After `push`, the function no longer has `config` -- it's been moved into the `Vec`. If `config` were borrowed (`&SnapshotTestingConfiguration`), the function would need to clone it before pushing. By taking ownership, the caller says "I'm done with this config, you can have it."

**Swift comparison:**
```swift
func withSnapshotTesting<R>(
    record: Record? = nil,
    diffTool: DiffTool? = nil,
    operation: () throws -> R
) rethrows -> R
```

In Swift, the `record` and `diffTool` parameters are value types -- they're copied into the function automatically. Rust makes you choose: borrow (cheap, temporary access) or move (transfer ownership permanently).

## The Ownership Decision Tree

When writing a Rust function, ask:

1. **Does this function need to store the value permanently?** -> Take ownership (`V`)
2. **Does it just need to read the value?** -> Borrow (`&V`)
3. **Does it need to modify the value?** -> Mutable borrow (`&mut V`)

In this project:
- `verify_snapshot` reads the value -> borrows (`&V`)
- `with_snapshot_testing` stores the config -> takes ownership (`config`)
- `pullback` stores the transform -> takes ownership (`transform`)

## Exercise

Run the exercise for this lesson:

```bash
cargo run -p snapshot-testing --example ex01_basic_diff
```

Open `crates/snapshot-testing/examples/ex01_basic_diff.rs` and follow the instructions in the comments. It walks you through:

- Calling `line_diff` with borrowed strings
- Seeing what happens if you try to use a moved value
- Writing a function that borrows vs one that takes ownership

## What's Next

In [Lesson 02: Lifetimes](./02-lifetimes.md), we'll see why the closures in `Snapshotting` and `Diffing` use `Arc<dyn Fn>` instead of references. The short answer: references have lifetimes, and lifetimes in struct fields create a cascade of complexity. `Arc` sidesteps that entirely.
