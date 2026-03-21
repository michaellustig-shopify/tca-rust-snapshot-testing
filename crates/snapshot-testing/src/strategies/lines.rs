//! +---------------------------------------------------------+
//! |  STRATEGY: LINES                                        |
//! |  Text line-based diffing for plain string snapshots     |
//! +---------------------------------------------------------+
//! |                                                         |
//! |   String ──► Snapshotting<String, String>               |
//! |                   │                                     |
//! |                   ▼                                     |
//! |             identity snapshot                           |
//! |                   │                                     |
//! |                   ▼                                     |
//! |             Diffing::lines()                            |
//! |             ├── to_data:   UTF-8 encode                 |
//! |             ├── from_data: UTF-8 decode                 |
//! |             └── diff:     line_diff (unified)           |
//! |                                                         |
//! +---------------------------------------------------------+
//! |  WHY: The most fundamental strategy. Almost every       |
//! |  other text-based strategy (json, debug, description)   |
//! |  builds on top of lines by pulling back through a       |
//! |  conversion function.                                   |
//! |                                                         |
//! |  SWIFT EQUIVALENT: Snapshotting<String, String>.lines   |
//! |  (Sources/SnapshotTesting/Snapshotting/Snapshotting.swift)
//! |                                                         |
//! |  REQUIREMENTS: REQ-STRAT-001, REQ-STRAT-002             |
//! |                                                         |
//! |  CHANGELOG:                                             |
//! |  * v0.1.0 -- Initial implementation                     |
//! +---------------------------------------------------------+

use crate::diff::line_diff;
use crate::diffing::{DiffAttachment, Diffing};
use crate::snapshotting::Snapshotting;

impl Diffing<String> {
    /// Create a line-based text diffing strategy.
    ///
    /// Serializes strings as UTF-8 bytes for disk storage, and compares them
    /// using a unified diff (the same format as `git diff`).
    ///
    /// When two strings differ, the diff output includes `@@` hunk markers
    /// and a `DiffAttachment` named `"difference.patch"` containing the raw
    /// patch text.
    ///
    /// # Requirements
    ///
    /// - REQ-STRAT-002a: Returns `None` when strings are identical.
    /// - REQ-STRAT-002b: Produces unified diff hunks with `@@` markers.
    /// - REQ-STRAT-002c: Includes a `DiffAttachment::Data` named `"difference.patch"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use snapshot_testing::Diffing;
    ///
    /// let diffing = Diffing::<String>::lines();
    ///
    /// // Identical strings produce no diff
    /// let result = (diffing.diff)(&"hello\n".to_string(), &"hello\n".to_string());
    /// assert!(result.is_none());
    ///
    /// // Different strings produce a unified diff with an attachment
    /// let result = (diffing.diff)(&"hello\n".to_string(), &"world\n".to_string());
    /// assert!(result.is_some());
    /// let (message, attachments) = result.unwrap();
    /// assert!(message.contains("@@"));
    /// assert_eq!(attachments.len(), 1);
    /// ```
    pub fn lines() -> Self {
        Diffing::new(
            // to_data: UTF-8 encode
            |s: &String| s.as_bytes().to_vec(),
            // from_data: UTF-8 decode (lossy for robustness)
            |data: &[u8]| String::from_utf8_lossy(data).into_owned(),
            // diff: line-level unified diff
            |old: &String, new: &String| {
                line_diff(old, new, 3).map(|diff_text| {
                    let attachment = DiffAttachment::Data {
                        bytes: diff_text.as_bytes().to_vec(),
                        name: "difference.patch".to_string(),
                    };
                    (diff_text, vec![attachment])
                })
            },
        )
    }
}

impl Snapshotting<String, String> {
    /// Create a line-based text snapshot strategy.
    ///
    /// This is the most fundamental strategy. The value (a `String`) is used
    /// directly as the snapshot format, and comparisons use line-level unified
    /// diffs. Stored on disk as `.txt` files encoded in UTF-8.
    ///
    /// Most other text-based strategies (`json`, `debug`, `description`) are
    /// built on top of this one via `pullback`.
    ///
    /// # Requirements
    ///
    /// - REQ-STRAT-001a: `path_extension` = `"txt"`
    /// - REQ-STRAT-001b: `to_data` = UTF-8 encode
    /// - REQ-STRAT-001c: `from_data` = UTF-8 decode
    /// - REQ-STRAT-001d: `diff` = line-level unified diff
    ///
    /// # Examples
    ///
    /// ```
    /// use snapshot_testing::Snapshotting;
    ///
    /// let strategy = Snapshotting::<String, String>::lines();
    /// assert_eq!(strategy.path_extension.as_deref(), Some("txt"));
    /// ```
    ///
    /// Using with a multi-line string:
    ///
    /// ```
    /// use snapshot_testing::Snapshotting;
    ///
    /// let strategy = Snapshotting::<String, String>::lines();
    /// let diffing = &strategy.diffing;
    ///
    /// let old = "line1\nline2\n".to_string();
    /// let new = "line1\nchanged\n".to_string();
    /// let result = (diffing.diff)(&old, &new);
    /// assert!(result.is_some());
    /// ```
    pub fn lines() -> Self {
        Snapshotting::new(
            Some("txt"),
            Diffing::<String>::lines(),
            |value: &String| std::future::ready(value.clone()),
        )
    }
}
