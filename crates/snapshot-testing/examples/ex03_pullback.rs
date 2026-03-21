// ============================================================================
// Exercise 03: Pullback — Composing Strategies
// Lessons: docs/curriculum/04-generics.md, docs/curriculum/06-closures.md
//
// Run with: cargo run -p snapshot-testing --example ex03_pullback
//
// This exercise teaches you:
//   1. What pullback does (transforms Snapshotting<A, F> into Snapshotting<B, F>)
//   2. How closures capture variables (the `move` keyword)
//   3. Chaining pullbacks to build complex strategies from simple ones
//   4. Generic functions with trait bounds
// ============================================================================

use snapshot_testing::{Diffing, Snapshotting};
use snapshot_testing::line_diff;

fn main() {
    println!("=== Exercise 03: Pullback ===\n");

    // -----------------------------------------------------------------------
    // Part 1: Start with a base strategy for String
    // -----------------------------------------------------------------------
    // The simplest strategy: snapshot a String as a String (identity).

    println!("--- Part 1: Base Strategy (String -> String) ---\n");

    let string_diffing = Diffing::<String>::new(
        |s: &String| s.as_bytes().to_vec(),
        |bytes: &[u8]| String::from_utf8_lossy(bytes).to_string(),
        |old: &String, new: &String| {
            line_diff(old, new, 3).map(|diff| (diff, vec![]))
        },
    );

    // Snapshotting<String, String> — takes a String, produces a String
    let lines_strategy: Snapshotting<String, String> = Snapshotting::new(
        Some("txt"),
        string_diffing.clone(),
        |s: &String| std::future::ready(s.clone()),  // identity: just clone the string
    );

    let input = "Hello, World!".to_string();
    let output = futures_block_on((lines_strategy.snapshot)(&input));
    println!("Base strategy: \"{}\" -> \"{}\"", input, output);
    println!();

    // -----------------------------------------------------------------------
    // Part 2: Pullback to i32 — make a strategy for numbers
    // -----------------------------------------------------------------------
    // pullback takes a Snapshotting<String, String> and a function i32 -> String
    // and produces a Snapshotting<i32, String>.
    //
    // The key insight: pullback goes BACKWARDS.
    // You have a strategy for String. You want a strategy for i32.
    // You provide: "how to turn an i32 into a String"
    // pullback does the rest.

    println!("--- Part 2: Pullback to i32 ---\n");

    let int_strategy: Snapshotting<i32, String> = lines_strategy.clone().pullback(
        |n: &i32| n.to_string(),  // transform: &i32 -> String
    );

    let number: i32 = 42;
    let output = futures_block_on((int_strategy.snapshot)(&number));
    println!("i32 strategy: {} -> \"{}\"", number, output);
    println!();

    // -----------------------------------------------------------------------
    // Part 3: Pullback to a struct — any type works
    // -----------------------------------------------------------------------

    println!("--- Part 3: Pullback to a Struct ---\n");

    #[derive(Debug)]
    #[allow(dead_code)]
    struct Product {
        name: String,
        price: f64,
    }

    // Snapshotting<Product, String> via pullback from the String strategy
    let product_strategy: Snapshotting<Product, String> = lines_strategy.clone().pullback(
        |p: &Product| format!("{} - ${:.2}", p.name, p.price),
    );

    let widget = Product {
        name: "Widget".to_string(),
        price: 9.99,
    };
    let output = futures_block_on((product_strategy.snapshot)(&widget));
    println!("Product strategy: {:?} -> \"{}\"", widget, output);

    // The original widget is still usable because pullback borrows (&Product)
    println!("Widget is still ours: {}", widget.name);
    println!();

    // -----------------------------------------------------------------------
    // Part 4: Chaining pullbacks
    // -----------------------------------------------------------------------
    // You can chain pullbacks to go deeper.
    //
    // String strategy -> i32 strategy -> Vec<i32> strategy
    //
    // Each pullback adds a transformation layer.

    println!("--- Part 4: Chaining Pullbacks ---\n");

    // Start: Snapshotting<String, String>
    // Step 1: pullback to Snapshotting<i32, String>
    // Step 2: pullback to Snapshotting<Vec<i32>, String>

    let vec_strategy: Snapshotting<Vec<i32>, String> = lines_strategy
        .clone()
        .pullback(|n: &i32| n.to_string())            // String <- i32
        .pullback(|v: &Vec<i32>| v.iter().sum::<i32>()); // i32 <- Vec<i32>

    let numbers = vec![1, 2, 3, 4, 5];
    let output = futures_block_on((vec_strategy.snapshot)(&numbers));
    println!("Vec<i32> strategy (sum): {:?} -> \"{}\"", numbers, output);

    // A different chain: format each element on its own line
    let vec_lines_strategy: Snapshotting<Vec<i32>, String> = lines_strategy
        .clone()
        .pullback(|v: &Vec<i32>| {
            v.iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        });

    let output = futures_block_on((vec_lines_strategy.snapshot)(&numbers));
    println!("Vec<i32> strategy (lines):");
    println!("{}\n", output);

    // -----------------------------------------------------------------------
    // Part 5: Pullback preserves path_extension and diffing
    // -----------------------------------------------------------------------

    println!("--- Part 5: Pullback Preserves Metadata ---\n");

    let json_diffing = Diffing::<String>::new(
        |s: &String| s.as_bytes().to_vec(),
        |bytes: &[u8]| String::from_utf8_lossy(bytes).to_string(),
        |old: &String, new: &String| {
            line_diff(old, new, 3).map(|diff| (diff, vec![]))
        },
    );

    let json_base: Snapshotting<String, String> = Snapshotting::new(
        Some("json"),       // path extension
        json_diffing,
        |s: &String| std::future::ready(s.clone()),
    );

    let json_for_int: Snapshotting<i32, String> = json_base.pullback(
        |n: &i32| format!("{{ \"value\": {} }}", n),
    );

    // The pullback preserved the "json" extension
    println!("Path extension after pullback: {:?}", json_for_int.path_extension);
    let output = futures_block_on((json_for_int.snapshot)(&42));
    println!("JSON int snapshot: {}", output);
    println!();

    // -----------------------------------------------------------------------
    // Part 6: Using the diff function
    // -----------------------------------------------------------------------

    println!("--- Part 6: Diffing After Pullback ---\n");

    let old_snapshot = "Widget - $9.99".to_string();
    let new_snapshot = "Widget - $12.99".to_string();

    let diff_result = (product_strategy.diffing.diff)(&old_snapshot, &new_snapshot);
    match diff_result {
        Some((diff_msg, _)) => {
            println!("Diff between old and new product snapshots:");
            println!("{}", diff_msg);
        }
        None => println!("No differences"),
    }

    // -----------------------------------------------------------------------
    // Part 7: YOUR TURN — Chain pullbacks for a nested type
    // -----------------------------------------------------------------------

    println!("--- Part 7: Your Turn ---\n");
    println!("TODO: Create a strategy for a nested type using pullback chains:");
    println!("  struct Order {{ product: Product, quantity: u32 }}");
    println!("Hint: pullback from lines_strategy, extracting fields from &Order");

    // Example solution (uncomment to try):
    //
    // struct Order { product: Product, quantity: u32 }
    //
    // let order_strategy: Snapshotting<Order, String> = lines_strategy.pullback(
    //     |order: &Order| format!(
    //         "{} - ${:.2} x {}",
    //         order.product.name, order.product.price, order.quantity
    //     ),
    // );
    //
    // let order = Order {
    //     product: Product { name: "Gadget".into(), price: 24.99 },
    //     quantity: 3,
    // };
    // let output = futures_block_on((order_strategy.snapshot)(&order));
    // println!("  Order snapshot: {}", output);

    println!("\n=== Exercise 03 Complete ===");
}

/// A minimal future executor for examples (same as ex02).
fn futures_block_on<F: std::future::Future>(future: F) -> F::Output {
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

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
