// ============================================================================
// Exercise 02: Custom Snapshotting Strategy
// Lessons: docs/curriculum/02-lifetimes.md, docs/curriculum/03-traits.md
//
// Run with: cargo run -p snapshot-testing --example ex02_custom_strategy
//
// This exercise teaches you:
//   1. How to create a Diffing<String> with Arc-wrapped closures
//   2. How to create a Snapshotting<MyType, String>
//   3. Why Arc makes these types cloneable
//   4. Why Snapshotting is a struct (not a trait) — multiple strategies per type
// ============================================================================

use snapshot_testing::{Diffing, Snapshotting};
use snapshot_testing::line_diff;

// A simple type to snapshot — notice it doesn't implement any snapshot-related trait
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct User {
    name: String,
    age: u32,
    email: String,
}

fn main() {
    println!("=== Exercise 02: Custom Snapshotting Strategy ===\n");

    // -----------------------------------------------------------------------
    // Part 1: Creating a Diffing<String>
    // -----------------------------------------------------------------------
    // Diffing needs three functions:
    //   to_data:   &String -> Vec<u8>      (serialize for disk)
    //   from_data: &[u8] -> String         (deserialize from disk)
    //   diff:      (&String, &String) -> Option<(String, Vec<DiffAttachment>)>

    println!("--- Part 1: Creating a Diffing<String> ---\n");

    let string_diffing = Diffing::<String>::new(
        // to_data: convert a String reference to bytes for storage
        |s: &String| s.as_bytes().to_vec(),

        // from_data: convert bytes back to a String
        |bytes: &[u8]| String::from_utf8_lossy(bytes).to_string(),

        // diff: compare two strings, return None if equal
        |old: &String, new: &String| {
            line_diff(old, new, 3).map(|diff| (diff, vec![]))
        },
    );

    println!("Created Diffing<String>: {:?}", string_diffing);

    // Clone it! This is cheap because closures are in Arc (just bumps ref counts)
    let _diffing_clone = string_diffing.clone();
    println!("Cloned Diffing<String> — Arc makes this cheap\n");

    // -----------------------------------------------------------------------
    // Part 2: Creating a Snapshotting<User, String> — the Debug strategy
    // -----------------------------------------------------------------------
    // This strategy snapshots a User by using its Debug output.

    println!("--- Part 2: Debug Strategy ---\n");

    let debug_strategy: Snapshotting<User, String> = Snapshotting::new(
        Some("txt"),                 // file extension for saved snapshots
        string_diffing.clone(),      // how to diff strings
        |user: &User| {
            // This closure borrows &User and returns a Future<Output = String>
            // std::future::ready wraps a value in an immediately-ready future
            let output = format!("{:#?}", user);  // Pretty Debug output
            std::future::ready(output)
        },
    );

    println!("Created Snapshotting<User, String> (debug): {:?}", debug_strategy);
    println!();

    // -----------------------------------------------------------------------
    // Part 3: Creating a DIFFERENT strategy for the SAME type
    // -----------------------------------------------------------------------
    // This is why Snapshotting is a struct, not a trait!
    // If it were a trait, User could only implement it once.
    // As a struct, we can create unlimited strategies.

    println!("--- Part 3: CSV Strategy (same User type, different format) ---\n");

    let csv_strategy: Snapshotting<User, String> = Snapshotting::new(
        Some("csv"),                // different file extension
        string_diffing.clone(),
        |user: &User| {
            let output = format!("{},{},{}", user.name, user.age, user.email);
            std::future::ready(output)
        },
    );

    println!("Created Snapshotting<User, String> (csv): {:?}", csv_strategy);
    println!();

    // A third strategy: JSON-like format
    let json_strategy: Snapshotting<User, String> = Snapshotting::new(
        Some("json"),
        string_diffing.clone(),
        |user: &User| {
            let output = format!(
                "{{\n  \"name\": \"{}\",\n  \"age\": {},\n  \"email\": \"{}\"\n}}",
                user.name, user.age, user.email
            );
            std::future::ready(output)
        },
    );

    println!("Created Snapshotting<User, String> (json): {:?}", json_strategy);
    println!("Three different strategies for the same User type!\n");

    // -----------------------------------------------------------------------
    // Part 4: Using the strategies (calling the snapshot function)
    // -----------------------------------------------------------------------

    println!("--- Part 4: Generating Snapshots ---\n");

    let user = User {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    };

    // The snapshot function is stored as Arc<dyn Fn(...)>
    // We call it like a regular function: (strategy.snapshot)(&user)
    // Then we block_on the future to get the result.
    let debug_output = futures_block_on((debug_strategy.snapshot)(&user));
    let csv_output = futures_block_on((csv_strategy.snapshot)(&user));
    let json_output = futures_block_on((json_strategy.snapshot)(&user));

    println!("Debug strategy output:");
    println!("{}\n", debug_output);

    println!("CSV strategy output:");
    println!("{}\n", csv_output);

    println!("JSON strategy output:");
    println!("{}\n", json_output);

    // -----------------------------------------------------------------------
    // Part 5: Cloning strategies (Arc makes this work)
    // -----------------------------------------------------------------------

    println!("--- Part 5: Cloning Strategies ---\n");

    let strategy_a = debug_strategy.clone();
    let strategy_b = debug_strategy.clone();

    // Both clones share the same underlying closure (via Arc)
    let out_a = futures_block_on((strategy_a.snapshot)(&user));
    let out_b = futures_block_on((strategy_b.snapshot)(&user));
    assert_eq!(out_a, out_b);
    println!("Two clones produce identical output (shared Arc closure)");
    println!();

    // -----------------------------------------------------------------------
    // Part 6: YOUR TURN — Create a strategy for a different type
    // -----------------------------------------------------------------------

    println!("--- Part 6: Your Turn ---\n");
    println!("TODO: Create a Snapshotting<Point, String> for this type:");
    println!("  struct Point {{ x: f64, y: f64 }}");
    println!("Hint: Use the same string_diffing, just write a new snapshot closure");
    println!("that formats the point however you like (e.g., \"(3.0, 4.0)\")");

    // Example solution (uncomment to try):
    //
    // #[derive(Debug)]
    // struct Point { x: f64, y: f64 }
    //
    // let point_strategy: Snapshotting<Point, String> = Snapshotting::new(
    //     Some("txt"),
    //     string_diffing.clone(),
    //     |p: &Point| std::future::ready(format!("({}, {})", p.x, p.y)),
    // );
    //
    // let p = Point { x: 3.0, y: 4.0 };
    // let output = futures_block_on((point_strategy.snapshot)(&p));
    // println!("  Point snapshot: {}", output);

    println!("\n=== Exercise 02 Complete ===");
}

/// A minimal future executor for examples.
/// In real code you'd use tokio or another async runtime.
/// This works because our futures are created with std::future::ready()
/// and resolve immediately.
fn futures_block_on<F: std::future::Future>(future: F) -> F::Output {
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

    // Create a no-op waker (we know the future is ready immediately)
    fn noop(_: *const ()) {}
    fn clone_noop(p: *const ()) -> RawWaker { RawWaker::new(p, &VTABLE) }
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone_noop, noop, noop, noop);

    let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
    let mut cx = Context::from_waker(&waker);
    let mut future = Box::pin(future);

    match future.as_mut().poll(&mut cx) {
        Poll::Ready(val) => val,
        Poll::Pending => panic!("Future was not immediately ready"),
    }
}
