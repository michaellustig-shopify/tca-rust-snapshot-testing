//! ┌─────────────────────────────────────────────────────┐
//! │  SNAPSHOTTING<V, F>                                  │
//! │  The core strategy abstraction — converts a value    │
//! │  of type V into a snapshot format F for comparison   │
//! ├─────────────────────────────────────────────────────┤
//! │                                                      │
//! │   Snapshotting<V, F>                                 │
//! │   ┌──────────────────────┐                           │
//! │   │ path_extension: &str │                           │
//! │   │ diffing: Diffing<F>  │                           │
//! │   │ snapshot: V → F      │                           │
//! │   └──────────┬───────────┘                           │
//! │              │                                       │
//! │   pullback:  │  NewV → V                             │
//! │   ┌──────────▼───────────┐                           │
//! │   │ Snapshotting<NewV, F>│                           │
//! │   └──────────────────────┘                           │
//! │                                                      │
//! ├─────────────────────────────────────────────────────┤
//! │  WHY: Separating "how to snapshot" from "what to     │
//! │  snapshot" makes the system composable. Any type     │
//! │  can be snapshot-tested by providing a strategy.     │
//! │                                                      │
//! │  ALTERNATIVES: Trait-based approach (impl Snapshot   │
//! │  for T). Rejected because strategies are more        │
//! │  flexible — same type can have multiple strategies.  │
//! │                                                      │
//! │  SWIFT EQUIVALENT: Snapshotting<Value, Format>       │
//! │  (Sources/SnapshotTesting/Snapshotting.swift)        │
//! │                                                      │
//! │  TESTED BY: tests/snapshotting_tests.rs              │
//! │  EDGE CASES: pullback composition chains, async      │
//! │  snapshot functions, format conversion failures      │
//! │                                                      │
//! │  CHANGELOG:                                          │
//! │  • v0.1.0 — Initial stub with core struct def        │
//! │                                                      │
//! │  HISTORY: git log --oneline --follow -- crates/snapshot-testing/src/snapshotting.rs
//! └─────────────────────────────────────────────────────┘

use crate::diffing::Diffing;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// The core snapshot strategy. Converts a value of type `V` into a format `F`
/// that can be diffed, serialized, and stored on disk.
///
/// # Type Parameters
///
/// - `V`: The value type being snapshotted (e.g., a struct, a view, a request)
/// - `F`: The format type for comparison (e.g., `String`, `Vec<u8>`, image bytes)
///
/// # Swift Equivalent
///
/// ```swift
/// struct Snapshotting<Value, Format> {
///     var pathExtension: String?
///     var diffing: Diffing<Format>
///     var snapshot: (Value) -> Async<Format>
/// }
/// ```
///
/// # Clone
///
/// Uses `Arc` for the snapshot closure so cloning is cheap.
/// See `Diffing` for the same pattern and rationale.
#[derive(Clone)]
pub struct Snapshotting<V, F>
where
    F: Clone,
{
    /// File extension for stored snapshots (e.g., "txt", "json", "png")
    pub path_extension: Option<String>,

    /// How to diff two values of the format type
    pub diffing: Diffing<F>,

    /// Function that converts a value into its snapshot format.
    /// Returns a future because some snapshots are async (e.g., rendering a view).
    pub snapshot: Arc<dyn Fn(&V) -> Pin<Box<dyn Future<Output = F> + Send>> + Send + Sync>,
}

impl<V, F> Snapshotting<V, F>
where
    F: Clone + Send + 'static,
    V: 'static,
{
    /// Create a new snapshotting strategy.
    ///
    /// # Arguments
    ///
    /// - `path_extension`: File extension for stored snapshots
    /// - `diffing`: How to compare two format values
    /// - `snapshot`: Function to convert a value to its snapshot format
    pub fn new<Snap, Fut>(
        path_extension: Option<&str>,
        diffing: Diffing<F>,
        snapshot: Snap,
    ) -> Self
    where
        Snap: Fn(&V) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = F> + Send + 'static,
    {
        Snapshotting {
            path_extension: path_extension.map(String::from),
            diffing,
            snapshot: Arc::new(move |v| Box::pin(snapshot(v))),
        }
    }

    /// Transform the input type. Given a function `NewV → V`, produces a
    /// `Snapshotting<NewV, F>` that first transforms then snapshots.
    ///
    /// This is the functional "pullback" — it lets you reuse existing
    /// strategies for new types by providing a conversion function.
    ///
    /// # Swift Equivalent
    ///
    /// ```swift
    /// func pullback<NewValue>(_ transform: @escaping (NewValue) -> Value)
    ///     -> Snapshotting<NewValue, Format>
    /// ```
    ///
    /// # Borrow Checker Note
    ///
    /// The transform function takes a reference `&NewV` and returns a
    /// *reference* to `V`. This avoids unnecessary cloning but requires
    /// the caller to ensure the returned reference is valid. For owned
    /// transforms, use `pullback_owned`.
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
}

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
