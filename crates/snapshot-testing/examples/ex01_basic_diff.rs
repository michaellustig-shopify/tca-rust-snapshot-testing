// ============================================================================
// Exercise 01: Basic Diff — Ownership and Borrowing
// Lesson: docs/curriculum/01-ownership-basics.md
//
// Run with: cargo run -p snapshot-testing --example ex01_basic_diff
//
// This exercise teaches you:
//   1. How line_diff borrows strings (takes &str, not String)
//   2. What happens when you try to use a moved value
//   3. How to write functions that borrow vs ones that take ownership
// ============================================================================

use snapshot_testing::line_diff;

fn main() {
    println!("=== Exercise 01: Basic Diff ===\n");

    // -----------------------------------------------------------------------
    // Part 1: Calling line_diff with borrowed strings
    // -----------------------------------------------------------------------
    // line_diff takes &str (borrowed string slices), not owned Strings.
    // This means we can keep using our strings after calling line_diff.

    let old_text = "hello\nworld\nfoo\n";
    let new_text = "hello\nearth\nfoo\n";

    // line_diff borrows old_text and new_text — it doesn't take ownership
    let result = line_diff(old_text, new_text, 3);

    // We can still use old_text and new_text here because they were borrowed, not moved
    println!("Old text: {:?}", old_text);
    println!("New text: {:?}", new_text);
    println!();

    match result {
        Some(diff) => {
            println!("Diff found:");
            println!("{}", diff);
        }
        None => {
            println!("No differences found.");
        }
    }

    // -----------------------------------------------------------------------
    // Part 2: When strings are identical, line_diff returns None
    // -----------------------------------------------------------------------

    let same_a = "identical\ncontent\n";
    let same_b = "identical\ncontent\n";

    let result = line_diff(same_a, same_b, 3);
    println!("Comparing identical strings:");
    match result {
        Some(diff) => println!("Unexpected diff: {}", diff),
        None => println!("  No differences (as expected)\n"),
    }

    // -----------------------------------------------------------------------
    // Part 3: Ownership vs Borrowing — A demonstration
    // -----------------------------------------------------------------------
    // Uncomment the section below to see a compile error about moved values.

    println!("--- Ownership Demo ---\n");

    // This function takes ownership (moves the String)
    fn consume_string(s: String) -> usize {
        println!("  Consumed string with {} bytes", s.len());
        s.len()
    }

    // This function borrows (takes a reference)
    fn borrow_string(s: &str) -> usize {
        println!("  Borrowed string with {} bytes", s.len());
        s.len()
    }

    let my_string = String::from("Hello, Rust!");

    // Borrowing: we can use my_string after this call
    let len1 = borrow_string(&my_string);
    println!("  After borrow, string is still ours: {:?}", my_string);
    println!("  Length: {}\n", len1);

    // Cloning: we can keep our copy by giving consume_string a clone
    let len2 = consume_string(my_string.clone());
    println!("  After consuming a clone, original is still ours: {:?}", my_string);
    println!("  Length: {}\n", len2);

    // Moving: after this, my_string is GONE
    let len3 = consume_string(my_string);
    println!("  Length: {}", len3);
    // Try uncommenting the next line — you'll get a compile error:
    // println!("After move: {:?}", my_string);  // ERROR: value used after move

    // -----------------------------------------------------------------------
    // Part 4: YOUR TURN — Write a function that borrows
    // -----------------------------------------------------------------------
    // Write a function called `count_lines` that:
    //   - Takes a &str (borrowed string slice)
    //   - Returns the number of lines in the string
    //   - Does NOT take ownership of the input
    //
    // Then call it below and verify that you can still use the input after.

    println!("\n--- Your Turn ---\n");
    println!("TODO: Implement count_lines(&str) -> usize");
    println!("Hint: Use .lines().count() on the string slice");

    // Example solution (uncomment to try):
    //
    // fn count_lines(text: &str) -> usize {
    //     text.lines().count()
    // }
    //
    // let poem = "Roses are red\nViolets are blue\nRust is great\nAnd so are you\n";
    // let count = count_lines(poem);
    // println!("  Lines: {}", count);
    // println!("  Original still available: {:?}", &poem[..14]);  // "Roses are red"

    println!("\n=== Exercise 01 Complete ===");
}
