# Lesson 00: Overview

## What This Project Is

This is a Rust port of [Point-Free's swift-snapshot-testing](https://github.com/pointfreeco/swift-snapshot-testing) library. The Swift version lets you write snapshot tests -- tests that capture a "picture" of your data and compare it against a saved reference. If the picture changes, the test fails and shows you what changed.

The key idea: instead of writing `assert_eq!(result, expected)` by hand, you let the library save the expected value to a file. Next time you run the test, it compares the fresh output to the saved file. If they differ, you get a diff (like `git diff`).

What makes Point-Free's library special is **strategies**. Instead of one way to snapshot (say, JSON), you can have many strategies for the same type: JSON, debug output, custom formats, images. Strategies are values you can compose and transform, not traits you implement once.

## Why Port From Swift to Rust?

Three reasons:

1. **The architecture is worth studying.** Point-Free designed a composable, strategy-based system that avoids the usual "implement this trait" approach. That same architecture works beautifully in Rust, but requires different tools (Arc instead of ARC, futures instead of callbacks, generics with trait bounds instead of protocol constraints).

2. **Rust doesn't have this exact pattern yet.** The `insta` crate is excellent but follows a different philosophy -- it's more opinionated and less composable. This port gives Rust developers the Point-Free approach.

3. **It teaches Rust idioms through real decisions.** Every "Why did you do it this way?" in this codebase has a concrete answer rooted in Rust's ownership system, trait system, or async model. That makes it a great teaching vehicle.

## The Crate Structure

The workspace has four crates (think of crates as Swift packages/modules):

| Crate | Swift Equivalent | What It Does |
|-------|-----------------|--------------|
| `snapshot-testing` | `SnapshotTesting` | Core engine: strategies, diffing, assertions, configuration |
| `inline-snapshot-testing` | `InlineSnapshotTesting` | Inline snapshots embedded in source code |
| `snapshot-testing-custom-dump` | `SnapshotTestingCustomDump` | Pretty-print strategy using structured debug output |
| `trinity` | *(no equivalent)* | CLI tool for managing the workspace |

The core crate (`snapshot-testing`) has these internal modules:

```
src/
  lib.rs            -- Re-exports everything
  snapshotting.rs   -- Snapshotting<V, F> struct (the strategy)
  diffing.rs        -- Diffing<V> struct (comparison engine)
  diff.rs           -- Line-based text diffing (wraps the `similar` crate)
  assert.rs         -- assert_snapshot / verify_snapshot
  config.rs         -- Record modes, DiffTool, thread-local configuration
```

## How This Curriculum Works

Each lesson teaches Rust concepts using **real code from this project**. Not contrived examples -- actual types, functions, and design decisions from the codebase.

**Format of each lesson:**

1. A Rust concept (e.g., ownership, lifetimes, traits)
2. Real code from this project that demonstrates it
3. Side-by-side Swift vs Rust comparisons
4. A "WHY" section explaining the design decision
5. An exercise you can compile and run

**Running exercises:**

Each exercise is a standalone Rust file in `crates/snapshot-testing/examples/`. Run them with:

```bash
cargo run -p snapshot-testing --example ex01_basic_diff
```

They print output to the terminal and have instructions in comments.

## Prerequisites

This curriculum assumes you:

- **Know Swift** (or another language with value types, closures, and generics)
- **Are new to Rust** (or still getting comfortable with ownership/borrowing)
- **Have Rust installed** (`rustup` and `cargo` work on your machine)
- **Can read code** -- you'll be reading real source files, not just lesson text

If you already know Rust well, you might still find the "WHY" sections interesting -- they explain why we made certain architectural choices that differ from typical Rust patterns.

## Lesson Map

| Lesson | Topic | Key Question |
|--------|-------|-------------|
| [01](./01-ownership-basics.md) | Ownership & Borrowing | Why does `verify_snapshot` take `&V` not `V`? |
| [02](./02-lifetimes.md) | Lifetimes | Why use `Arc<dyn Fn>` instead of `&'a dyn Fn`? |
| [03](./03-traits.md) | Traits | Why is `Snapshotting` a struct, not a trait? |
| [04](./04-generics.md) | Generics | What do `<V, F>` and trait bounds mean? |
| [05](./05-error-handling.md) | Error Handling | Why does `verify_snapshot` return `Result`? |
| [06](./06-closures.md) | Closures | Why `Arc<dyn Fn>` not `fn` pointers? |
| [07](./07-async.md) | Async Rust | Why `Pin<Box<dyn Future>>`? |
| [08](./08-testing.md) | Testing | How do `#[test]`, doc tests, and `insta` work? |
| [09](./09-modules.md) | Modules & Crates | Why split into 4 crates? |
| [10](./10-advanced.md) | Advanced Topics | Thread-locals, `Send + Sync`, interior mutability |

Start with [Lesson 01: Ownership Basics](./01-ownership-basics.md).
