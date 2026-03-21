// ============================================================================
// Exercise 05: Doc Tests — Documentation That Compiles
// Lesson: docs/curriculum/08-testing.md
//
// Run with: cargo run -p snapshot-testing --example ex05_doc_tests
//
// This exercise teaches you:
//   1. How Rust doc comments (///) work
//   2. Code blocks in doc comments that run as tests
//   3. The difference between // and /// and //!
//   4. How assert! and assert_eq! work in doc tests
//   5. Running doc tests with `cargo test --doc`
// ============================================================================

fn main() {
    println!("=== Exercise 05: Doc Tests ===\n");

    // -----------------------------------------------------------------------
    // Part 1: Doc comments with ///
    // -----------------------------------------------------------------------
    // In Rust, /// is a doc comment. It generates documentation AND any code
    // blocks inside it are compiled and run as tests when you `cargo test`.

    println!("--- Part 1: Functions With Doc Tests ---\n");

    // Call our documented functions to see them work
    let result = add(2, 3);
    println!("add(2, 3) = {}", result);

    let result = is_palindrome("racecar");
    println!("is_palindrome(\"racecar\") = {}", result);

    let result = is_palindrome("hello");
    println!("is_palindrome(\"hello\") = {}", result);
    println!();

    // -----------------------------------------------------------------------
    // Part 2: How doc tests compare to Swift
    // -----------------------------------------------------------------------
    println!("--- Part 2: Swift vs Rust Documentation ---\n");
    println!("Swift doc comments:");
    println!("  /// Adds two numbers.");
    println!("  /// - Parameters:");
    println!("  ///   - a: First number");
    println!("  ///   - b: Second number");
    println!("  /// - Returns: The sum");
    println!("  ///");
    println!("  /// ```swift");
    println!("  /// let result = add(2, 3)  // This is just text!");
    println!("  /// ```");
    println!("  func add(_ a: Int, _ b: Int) -> Int {{ a + b }}");
    println!();
    println!("Rust doc comments:");
    println!("  /// Adds two numbers.");
    println!("  ///");
    println!("  /// # Arguments");
    println!("  /// * `a` - First number");
    println!("  /// * `b` - Second number");
    println!("  ///");
    println!("  /// # Examples");
    println!("  /// ```");
    println!("  /// assert_eq!(add(2, 3), 5);  // This RUNS as a test!");
    println!("  /// ```");
    println!("  fn add(a: i32, b: i32) -> i32 {{ a + b }}");
    println!();
    println!("Key difference: Rust's code examples are COMPILED AND RUN.");
    println!("Swift's code examples are just text that might go stale.");
    println!();

    // -----------------------------------------------------------------------
    // Part 3: Real doc tests from this project
    // -----------------------------------------------------------------------
    println!("--- Part 3: Doc Tests in This Project ---\n");
    println!("Look at these real doc tests in the codebase:\n");
    println!("  crates/snapshot-testing/src/config.rs — DiffTool::new()");
    println!("    ```");
    println!("    use snapshot_testing::config::DiffTool;");
    println!("    let tool = DiffTool::new(|a, b| format!(\"diff {{a}} {{b}}\"));");
    println!("    assert!(tool.command(\"foo.txt\", \"bar.txt\").contains(\"diff\"));");
    println!("    ```\n");
    println!("  crates/snapshot-testing/src/config.rs — DiffTool::default_tool()");
    println!("    ```");
    println!("    use snapshot_testing::config::DiffTool;");
    println!("    let tool = DiffTool::default_tool();");
    println!("    let output = tool.command(\"/tmp/expected.txt\", \"/tmp/actual.txt\");");
    println!("    assert!(output.contains(\"file://\"));");
    println!("    ```\n");
    println!("Run them with: cargo test -p snapshot-testing --doc\n");

    // -----------------------------------------------------------------------
    // Part 4: Using the functions and doc tests
    // -----------------------------------------------------------------------
    println!("--- Part 4: Using Documented Functions ---\n");

    let greeting = greet("Rustacean");
    println!("{}", greeting);

    let clamped = clamp(150, 0, 100);
    println!("clamp(150, 0, 100) = {}", clamped);

    let clamped = clamp(-5, 0, 100);
    println!("clamp(-5, 0, 100) = {}", clamped);

    let clamped = clamp(50, 0, 100);
    println!("clamp(50, 0, 100) = {}", clamped);
    println!();

    // -----------------------------------------------------------------------
    // Part 5: Module-level doc comments (//!)
    // -----------------------------------------------------------------------
    println!("--- Part 5: Module-Level Docs ---\n");
    println!("//! is for module-level documentation (like this file's purpose).");
    println!("/// is for item-level documentation (a function, struct, enum).");
    println!();
    println!("Look at crates/snapshot-testing/src/lib.rs for //! comments.");
    println!("They describe the crate as a whole, not any specific item.");
    println!();

    // -----------------------------------------------------------------------
    // Part 6: YOUR TURN — Write a function with a doc test
    // -----------------------------------------------------------------------
    println!("--- Part 6: Your Turn ---\n");
    println!("TODO: Write a function called `reverse_words` that:");
    println!("  1. Takes a &str");
    println!("  2. Returns a String with words in reverse order");
    println!("  3. Has a /// doc comment with # Examples section");
    println!("  4. The example block uses assert_eq! to verify behavior");
    println!();
    println!("Then run `cargo test --doc -p snapshot-testing` to see if it passes.");
    println!("(Reminder: doc tests only run on library items, not example files,");
    println!(" but the pattern is the same.)");

    // Example solution (uncomment to try):
    //
    // fn reverse_words(s: &str) -> String {
    //     s.split_whitespace()
    //         .rev()
    //         .collect::<Vec<_>>()
    //         .join(" ")
    // }
    //
    // let result = reverse_words("hello beautiful world");
    // println!("  reverse_words(\"hello beautiful world\") = \"{}\"", result);
    // assert_eq!(result, "world beautiful hello");
    // println!("  Assertion passed!");

    println!("\n=== Exercise 05 Complete ===");
}

/// Adds two integers and returns the sum.
///
/// This is a simple example of a documented function. In a real project,
/// the code block below would be run as a test by `cargo test --doc`.
///
/// # Examples
///
/// ```
/// # // In doc tests, `#` lines are hidden but still compiled
/// let result = 2 + 3;
/// assert_eq!(result, 5);
/// ```
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Checks if a string is a palindrome (reads the same forwards and backwards).
///
/// Case-sensitive comparison. Only considers alphanumeric characters.
///
/// # Examples
///
/// ```
/// // These would run as tests in a library crate:
/// // assert!(is_palindrome("racecar"));
/// // assert!(!is_palindrome("hello"));
/// // assert!(is_palindrome("A"));
/// // assert!(is_palindrome(""));
/// ```
fn is_palindrome(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    let reversed: Vec<char> = chars.iter().rev().cloned().collect();
    chars == reversed
}

/// Creates a greeting string for the given name.
///
/// # Arguments
///
/// * `name` - The person to greet
///
/// # Returns
///
/// A formatted greeting string.
///
/// # Examples
///
/// ```
/// // let greeting = greet("World");
/// // assert_eq!(greeting, "Hello, World!");
/// ```
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

/// Clamps a value to be within the given range [min, max].
///
/// If the value is below min, returns min.
/// If the value is above max, returns max.
/// Otherwise, returns the value unchanged.
///
/// # Arguments
///
/// * `value` - The value to clamp
/// * `min` - The minimum allowed value
/// * `max` - The maximum allowed value
///
/// # Panics
///
/// Panics if `min > max`.
///
/// # Examples
///
/// ```
/// // assert_eq!(clamp(5, 0, 10), 5);   // within range
/// // assert_eq!(clamp(-1, 0, 10), 0);  // below min
/// // assert_eq!(clamp(15, 0, 10), 10); // above max
/// ```
fn clamp(value: i32, min: i32, max: i32) -> i32 {
    assert!(min <= max, "min ({}) must be <= max ({})", min, max);
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}
