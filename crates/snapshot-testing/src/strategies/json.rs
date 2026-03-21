//! +---------------------------------------------------------+
//! |  STRATEGY: JSON                                         |
//! |  Serialize values as pretty-printed JSON for diffing    |
//! +---------------------------------------------------------+
//! |                                                         |
//! |   T: Serialize ──► Snapshotting<T, String>              |
//! |                        │                                |
//! |                        ▼                                |
//! |               serde_json::to_string_pretty              |
//! |                        │                                |
//! |                        ▼                                |
//! |               Diffing::lines() (text diff)              |
//! |                                                         |
//! +---------------------------------------------------------+
//! |  WHY: JSON is the most common serialization format.     |
//! |  Pretty-printing with sorted keys ensures deterministic |
//! |  output so snapshots don't flap due to key ordering.    |
//! |                                                         |
//! |  SWIFT EQUIVALENT: Snapshotting<Value, String>.json     |
//! |  where Value: Encodable                                 |
//! |                                                         |
//! |  REQUIREMENTS: REQ-STRAT-005                            |
//! |                                                         |
//! |  NOTE: serde_json already sorts keys when you use       |
//! |  to_string_pretty on types with BTreeMap. For HashMap,  |
//! |  we serialize to serde_json::Value first (which uses    |
//! |  BTreeMap internally via Map) to get sorted keys.       |
//! |                                                         |
//! |  CHANGELOG:                                             |
//! |  * v0.1.0 -- Initial implementation                     |
//! +---------------------------------------------------------+

use crate::diffing::Diffing;
use crate::snapshotting::Snapshotting;
use serde::Serialize;

impl<V> Snapshotting<V, String>
where
    V: Serialize + 'static,
{
    /// Create a JSON snapshot strategy.
    ///
    /// Serializes values to pretty-printed JSON with sorted keys, then diffs
    /// the resulting text using line-level unified diffs. Stored on disk as
    /// `.json` files.
    ///
    /// Sorted keys are achieved by first serializing to `serde_json::Value`
    /// (which uses a `BTreeMap` for object keys), then pretty-printing. This
    /// ensures deterministic output regardless of the source type's field or
    /// key ordering.
    ///
    /// # Requirements
    ///
    /// - REQ-STRAT-005a: `path_extension` = `"json"`
    /// - REQ-STRAT-005b: Pretty-printed with sorted keys
    ///
    /// # Panics
    ///
    /// The snapshot closure panics if serialization fails. This is intentional:
    /// a type that claims to be `Serialize` but fails at runtime is a bug in
    /// the test, not a snapshot mismatch.
    ///
    /// # Examples
    ///
    /// ```
    /// use snapshot_testing::Snapshotting;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct User {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// let strategy = Snapshotting::<User, String>::json();
    /// assert_eq!(strategy.path_extension.as_deref(), Some("json"));
    /// ```
    ///
    /// The snapshot output is pretty-printed JSON:
    ///
    /// ```
    /// use snapshot_testing::Snapshotting;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Point { x: i32, y: i32 }
    ///
    /// let strategy = Snapshotting::<Point, String>::json();
    ///
    /// // Verify diffing round-trips through UTF-8
    /// let data = (strategy.diffing.to_data)(&"{\"x\":1}".to_string());
    /// let back = (strategy.diffing.from_data)(&data);
    /// assert_eq!(back, "{\"x\":1}");
    /// ```
    pub fn json() -> Self {
        Snapshotting::new(
            Some("json"),
            Diffing::<String>::lines(),
            |value: &V| {
                // Serialize to serde_json::Value first to get sorted keys
                // (serde_json::Value uses BTreeMap for Map, which sorts keys)
                let json_value = serde_json::to_value(value)
                    .expect("Failed to serialize value to JSON");
                let pretty = serde_json::to_string_pretty(&json_value)
                    .expect("Failed to pretty-print JSON value");
                std::future::ready(pretty)
            },
        )
    }
}
