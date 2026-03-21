//! +---------------------------------------------------------+
//! |  STRATEGY: DESCRIPTION                                  |
//! |  Snapshot values using Rust's Display trait              |
//! +---------------------------------------------------------+
//! |                                                         |
//! |   T: Display ──► Snapshotting<T, String>                |
//! |                       │                                 |
//! |                       ▼                                 |
//! |              value.to_string()                          |
//! |                       │                                 |
//! |                       ▼                                 |
//! |              Diffing::lines() (text diff)               |
//! |                                                         |
//! +---------------------------------------------------------+
//! |  WHY: Display is Rust's equivalent of Swift's           |
//! |  String(describing:). It produces the "user-facing"     |
//! |  representation of a value, as opposed to Debug which   |
//! |  produces the "programmer-facing" representation.       |
//! |                                                         |
//! |  Use this for types that have a meaningful Display      |
//! |  impl (e.g., error messages, formatted output, URLs).   |
//! |  Use `debug()` for types where you want structural      |
//! |  introspection.                                         |
//! |                                                         |
//! |  SWIFT EQUIVALENT:                                      |
//! |  Snapshotting<Value, String>.description                |
//! |  (uses String(describing:))                             |
//! |                                                         |
//! |  REQUIREMENTS: REQ-STRAT-003                            |
//! |                                                         |
//! |  CHANGELOG:                                             |
//! |  * v0.1.0 -- Initial implementation                     |
//! +---------------------------------------------------------+

use crate::diffing::Diffing;
use crate::snapshotting::Snapshotting;
use std::fmt::Display;

impl<V> Snapshotting<V, String>
where
    V: Display + 'static,
{
    /// Create a Display-based snapshot strategy.
    ///
    /// Converts values to their `Display` representation using `to_string()`,
    /// then diffs the resulting text using line-level unified diffs. Stored on
    /// disk as `.txt` files.
    ///
    /// This is the Rust equivalent of Swift's `Snapshotting.description` which
    /// uses `String(describing:)`. Use it for types that have a meaningful
    /// `Display` impl.
    ///
    /// # Requirements
    ///
    /// - REQ-STRAT-003a: Uses `Display` trait (Swift's `String(describing:)`).
    /// - REQ-STRAT-003b: Pulls back through `Display::to_string`.
    ///
    /// # Examples
    ///
    /// ```
    /// use snapshot_testing::Snapshotting;
    /// use std::fmt;
    ///
    /// struct Greeting {
    ///     name: String,
    /// }
    ///
    /// impl fmt::Display for Greeting {
    ///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         write!(f, "Hello, {}!", self.name)
    ///     }
    /// }
    ///
    /// let strategy = Snapshotting::<Greeting, String>::description();
    /// assert_eq!(strategy.path_extension.as_deref(), Some("txt"));
    /// ```
    ///
    /// Works with standard library types that implement Display:
    ///
    /// ```
    /// use snapshot_testing::Snapshotting;
    ///
    /// // i32 implements Display
    /// let strategy = Snapshotting::<i32, String>::description();
    /// assert_eq!(strategy.path_extension.as_deref(), Some("txt"));
    ///
    /// // Diffing works through lines
    /// let a = "42".to_string();
    /// let b = "99".to_string();
    /// let result = (strategy.diffing.diff)(&a, &b);
    /// assert!(result.is_some());
    /// ```
    pub fn description() -> Self {
        Snapshotting::new(
            Some("txt"),
            Diffing::<String>::lines(),
            |value: &V| {
                let formatted = value.to_string();
                std::future::ready(formatted)
            },
        )
    }
}
