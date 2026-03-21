# Exercises

These exercises are runnable Rust programs that accompany the curriculum lessons.

## How to Run

Each exercise is registered as an example in the `snapshot-testing` crate. Run them with:

```bash
cargo run -p snapshot-testing --example <exercise_name>
```

## Exercise List

| Exercise | File | Lesson | What You'll Learn |
|----------|------|--------|-------------------|
| ex01_basic_diff | `crates/snapshot-testing/examples/ex01_basic_diff.rs` | [01 - Ownership](../01-ownership-basics.md) | Borrowing with `line_diff`, moved values, writing functions that borrow vs own |
| ex02_custom_strategy | `crates/snapshot-testing/examples/ex02_custom_strategy.rs` | [02 - Lifetimes](../02-lifetimes.md), [03 - Traits](../03-traits.md) | Creating `Diffing<String>`, building `Snapshotting<MyType, String>`, multiple strategies per type |
| ex03_pullback | `crates/snapshot-testing/examples/ex03_pullback.rs` | [04 - Generics](../04-generics.md), [06 - Closures](../06-closures.md) | Pullback composition, closure captures, chaining strategies |
| ex04_record_modes | `crates/snapshot-testing/examples/ex04_record_modes.rs` | [05 - Errors](../05-error-handling.md), [10 - Advanced](../10-advanced.md) | Record modes, `with_snapshot_testing` nesting, `Result` pattern matching |
| ex05_doc_tests | `crates/snapshot-testing/examples/ex05_doc_tests.rs` | [08 - Testing](../08-testing.md) | Doc comments, code examples that compile, the `///` syntax |

## Exercise Structure

Each exercise has:

1. **Guided parts** — Code that runs and prints results, with comments explaining what's happening
2. **A "Your Turn" section** — A TODO at the end where you write code yourself
3. **A commented-out solution** — Uncomment to check your work

## Tips

- Read the lesson before running the exercise
- Read the actual source files referenced in each exercise (in `crates/snapshot-testing/src/`)
- Try modifying the exercises to experiment with the concepts
- If something doesn't compile, read the error message carefully -- Rust's error messages are very helpful
