//! Tests ported from WaitTests.swift
//!
//! The Swift `WaitTests` verified an async snapshot strategy: `.wait(for:on:)`
//! which delays taking the snapshot until an async operation completes.
//! This is useful when snapshotting values that change over time (e.g.,
//! a value updated by a DispatchQueue.asyncAfter).
//!
//! In Rust, the equivalent would be an async snapshotting strategy that
//! waits for a duration before capturing.

#[cfg(test)]
mod wait_tests {
    #[allow(unused_imports)]
    use snapshot_testing::Snapshotting;

    /// Verifies that the `.wait(for:on:)` strategy delays snapshot capture.
    ///
    /// The Swift test:
    /// 1. Sets `value = "Hello"`
    /// 2. Schedules `value = "Goodbye"` after 1 second
    /// 3. Uses `.wait(for: 1.5, on: .lines)` to snapshot after 1.5 seconds
    /// 4. The snapshot should capture "Goodbye" (the updated value)
    ///
    /// In Rust, this would use `tokio::time::sleep` or similar async delay.
    ///
    /// Swift: `func testWait()`
    #[test]
    #[ignore] // TODO: implement Snapshotting::wait() async delay strategy
    fn test_wait() {
        // let mut value = String::from("Hello");
        //
        // // Simulate async update (in Rust, use tokio::spawn or similar)
        // // After 1 second: value = "Goodbye"
        //
        // // The .wait strategy pulls back through .lines and adds a delay
        // let strategy = Snapshotting::<(), String>::wait(
        //     Duration::from_secs_f64(1.5),
        //     Snapshotting::lines().pullback(|_: &()| value.clone()),
        // );
        //
        // assert_snapshot(&(), &strategy, None, ...);
        // // Snapshot should contain "Goodbye"
    }
}
