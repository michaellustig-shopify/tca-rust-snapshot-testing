//! +---------------------------------------------------------+
//! |  STRATEGY: DEBUG                                        |
//! |  Snapshot values using Rust's Debug trait               |
//! +---------------------------------------------------------+
//! |                                                         |
//! |   T: Debug ──► Snapshotting<T, String>                  |
//! |                     │                                   |
//! |                     ▼                                   |
//! |            format!("{:#?}", value)                      |
//! |                     │                                   |
//! |                     ▼                                   |
//! |            Diffing::lines() (text diff)                 |
//! |                                                         |
//! +---------------------------------------------------------+
//! |  WHY: Debug is Rust's closest equivalent to Swift's     |
//! |  `dump()` / Mirror-based introspection. Almost every    |
//! |  type derives Debug, making this the most universally   |
//! |  applicable strategy. Uses `{:#?}` (pretty Debug) for  |
//! |  readable, indented output.                             |
//! |                                                         |
//! |  SWIFT EQUIVALENT: Snapshotting<Value, String>.dump     |
//! |  (uses CustomDump / Mirror)                             |
//! |                                                         |
//! |  REQUIREMENTS: REQ-STRAT-004                            |
//! |                                                         |
//! |  CHANGELOG:                                             |
//! |  * v0.1.0 -- Initial implementation                     |
//! +---------------------------------------------------------+

use crate::diffing::Diffing;
use crate::snapshotting::Snapshotting;
use std::fmt::Debug;

impl<V> Snapshotting<V, String>
where
    V: Debug + 'static,
{
    /// Create a Debug-based snapshot strategy.
    ///
    /// Converts values to their pretty-printed Debug representation using
    /// `format!("{:#?}", value)`, then diffs the resulting text using
    /// line-level unified diffs. Stored on disk as `.txt` files.
    ///
    /// This is the most universally applicable strategy since nearly every
    /// Rust type derives `Debug`. The `{:#?}` format produces indented,
    /// multi-line output that diffs cleanly.
    ///
    /// # Requirements
    ///
    /// - REQ-STRAT-004a: Uses `Debug` trait (Rust equivalent of Swift's `dump`).
    /// - REQ-STRAT-004b: Uses `format!("{:#?}", value)` for pretty-printed debug.
    ///
    /// # Examples
    ///
    /// ```
    /// use snapshot_testing::Snapshotting;
    ///
    /// #[derive(Debug)]
    /// struct Config {
    ///     host: String,
    ///     port: u16,
    /// }
    ///
    /// let strategy = Snapshotting::<Config, String>::debug();
    /// assert_eq!(strategy.path_extension.as_deref(), Some("txt"));
    /// ```
    ///
    /// The output uses Rust's pretty Debug format:
    ///
    /// ```
    /// use snapshot_testing::Snapshotting;
    ///
    /// let strategy = Snapshotting::<Vec<i32>, String>::debug();
    ///
    /// // Diffing works through lines
    /// let a = "[1, 2, 3]".to_string();
    /// let b = "[1, 2, 4]".to_string();
    /// let result = (strategy.diffing.diff)(&a, &b);
    /// assert!(result.is_some());
    /// ```
    pub fn debug() -> Self {
        Snapshotting::new(
            Some("txt"),
            Diffing::<String>::lines(),
            |value: &V| {
                let formatted = format!("{:#?}", value);
                std::future::ready(formatted)
            },
        )
    }
}
