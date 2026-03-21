//! ┌─────────────────────────────────────────────────────┐
//! │  DIFFING<V>                                          │
//! │  Comparison + serialization for snapshot formats     │
//! ├─────────────────────────────────────────────────────┤
//! │                                                      │
//! │    Value A ──┐                                       │
//! │              ├──► diff() ──► Option<(String, Vec)>   │
//! │    Value B ──┘                                       │
//! │                                                      │
//! │    Value ──► to_data() ──► Vec<u8>                   │
//! │    Vec<u8> ──► from_data() ──► Value                 │
//! │                                                      │
//! ├─────────────────────────────────────────────────────┤
//! │  WHY: Separating diffing from snapshotting allows    │
//! │  reuse. A single Diffing<String> works for JSON,     │
//! │  plain text, XML — anything that ends up as text.    │
//! │                                                      │
//! │  SWIFT EQUIVALENT: Diffing<Value>                    │
//! │  (Sources/SnapshotTesting/Diffing.swift)             │
//! │                                                      │
//! │  TESTED BY: tests/diffing_tests.rs                   │
//! │  EDGE CASES: identical values, empty values, binary  │
//! │  data with no text representation                    │
//! │                                                      │
//! │  CHANGELOG:                                          │
//! │  • v0.1.0 — Initial stub                             │
//! │                                                      │
//! │  HISTORY: git log --oneline --follow -- crates/snapshot-testing/src/diffing.rs
//! └─────────────────────────────────────────────────────┘

use std::sync::Arc;

/// Attachment produced during a diff — either raw data or a named blob.
///
/// # Swift Equivalent
///
/// ```swift
/// enum DiffAttachment {
///     case data(Data, name: String)
/// }
/// ```
#[derive(Debug, Clone)]
pub enum DiffAttachment {
    /// Raw bytes with a human-readable name (e.g., "expected.png", "actual.png")
    Data { bytes: Vec<u8>, name: String },
}

/// How to compare and serialize two values of the same type.
///
/// `Diffing` is the comparison engine. It knows how to:
/// 1. Convert a value to bytes (for disk storage)
/// 2. Convert bytes back to a value (for loading from disk)
/// 3. Compare two values and produce a human-readable diff
///
/// # Type Parameter
///
/// - `V`: The format type being compared (e.g., `String`, `Vec<u8>`)
///
/// # Swift Equivalent
///
/// ```swift
/// struct Diffing<Value> {
///     var toData: (Value) -> Data
///     var fromData: (Data) -> Value
///     var diff: (Value, Value) -> (String, [DiffAttachment])?
/// }
/// ```
///
/// # Lifetime Note
///
/// The closures are boxed and owned because `Diffing` instances are stored
/// inside `Snapshotting` and must outlive any particular comparison call.
/// We use `Box<dyn Fn>` rather than generics to keep the struct object-safe
/// and storable in collections.
/// # Clone
///
/// Uses `Arc` internally so cloning is cheap (reference count bump).
/// This is necessary because `Box<dyn Fn>` doesn't implement `Clone`,
/// but we need `Diffing` to be cloneable since it's stored inside
/// `Snapshotting` which may be shared across tests.
#[derive(Clone)]
pub struct Diffing<V>
where
    V: Clone,
{
    /// Serialize a value to bytes for disk storage
    pub to_data: Arc<dyn Fn(&V) -> Vec<u8> + Send + Sync>,

    /// Deserialize bytes from disk back into a value
    pub from_data: Arc<dyn Fn(&[u8]) -> V + Send + Sync>,

    /// Compare two values. Returns `None` if equal, or `Some((message, attachments))`
    /// with a human-readable diff description and optional binary attachments.
    pub diff: Arc<dyn Fn(&V, &V) -> Option<(String, Vec<DiffAttachment>)> + Send + Sync>,
}

impl<V: Clone> Diffing<V> {
    /// Create a new `Diffing` with the provided serialization and comparison functions.
    ///
    /// # Arguments
    ///
    /// - `to_data`: Converts a value to bytes for storage
    /// - `from_data`: Converts bytes back to a value
    /// - `diff`: Compares two values, returns `None` if equal
    pub fn new<TD, FD, D>(to_data: TD, from_data: FD, diff: D) -> Self
    where
        TD: Fn(&V) -> Vec<u8> + Send + Sync + 'static,
        FD: Fn(&[u8]) -> V + Send + Sync + 'static,
        D: Fn(&V, &V) -> Option<(String, Vec<DiffAttachment>)> + Send + Sync + 'static,
    {
        Diffing {
            to_data: Arc::new(to_data),
            from_data: Arc::new(from_data),
            diff: Arc::new(diff),
        }
    }
}

impl<V: Clone> std::fmt::Debug for Diffing<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Diffing").finish_non_exhaustive()
    }
}
