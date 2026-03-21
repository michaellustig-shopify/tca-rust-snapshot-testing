//! +---------------------------------------------------------+
//! |  STRATEGY: DATA                                         |
//! |  Raw binary data comparison                             |
//! +---------------------------------------------------------+
//! |                                                         |
//! |   Vec<u8> ──► Snapshotting<Vec<u8>, Vec<u8>>            |
//! |                    │                                    |
//! |                    ▼                                    |
//! |             identity snapshot                           |
//! |                    │                                    |
//! |                    ▼                                    |
//! |             Diffing<Vec<u8>>                            |
//! |             ├── to_data:   identity (already bytes)     |
//! |             ├── from_data: identity (already bytes)     |
//! |             └── diff:     byte-by-byte comparison       |
//! |                           reports first difference pos  |
//! |                                                         |
//! +---------------------------------------------------------+
//! |  WHY: Some values are inherently binary (images, audio, |
//! |  protobuf, etc.). This strategy compares raw bytes and  |
//! |  reports where the first difference occurs.             |
//! |                                                         |
//! |  SWIFT EQUIVALENT: Snapshotting<Data, Data>.data        |
//! |                                                         |
//! |  REQUIREMENTS: REQ-STRAT-006                            |
//! |                                                         |
//! |  CHANGELOG:                                             |
//! |  * v0.1.0 -- Initial implementation                     |
//! +---------------------------------------------------------+

use crate::diffing::Diffing;
use crate::snapshotting::Snapshotting;

impl Diffing<Vec<u8>> {
    /// Create a byte-level binary data diffing strategy.
    ///
    /// Compares raw byte sequences and reports the position of the first
    /// difference. Serialization is identity (bytes are already bytes).
    ///
    /// # Examples
    ///
    /// ```
    /// use snapshot_testing::Diffing;
    ///
    /// let diffing = Diffing::<Vec<u8>>::data();
    ///
    /// // Identical data produces no diff
    /// let result = (diffing.diff)(&vec![1, 2, 3], &vec![1, 2, 3]);
    /// assert!(result.is_none());
    ///
    /// // Different data reports the position
    /// let result = (diffing.diff)(&vec![1, 2, 3], &vec![1, 2, 4]);
    /// assert!(result.is_some());
    /// let (message, _) = result.unwrap();
    /// assert!(message.contains("byte 2"));
    /// ```
    pub fn data() -> Self {
        Diffing::new(
            // to_data: identity -- already bytes
            |v: &Vec<u8>| v.clone(),
            // from_data: identity -- already bytes
            |data: &[u8]| data.to_vec(),
            // diff: byte-by-byte comparison
            |old: &Vec<u8>, new: &Vec<u8>| {
                if old == new {
                    return None;
                }

                let mut message = String::new();

                // Report size difference
                if old.len() != new.len() {
                    message.push_str(&format!(
                        "Data size mismatch: expected {} bytes, got {} bytes.\n",
                        old.len(),
                        new.len()
                    ));
                }

                // Find and report position of first difference
                let first_diff = old
                    .iter()
                    .zip(new.iter())
                    .position(|(a, b)| a != b);

                match first_diff {
                    Some(pos) => {
                        message.push_str(&format!(
                            "First difference at byte {}: expected 0x{:02X}, got 0x{:02X}.",
                            pos, old[pos], new[pos]
                        ));
                    }
                    None => {
                        // Lengths differ but overlapping prefix is the same
                        let shorter = old.len().min(new.len());
                        message.push_str(&format!(
                            "Data matches for the first {} bytes, then diverges (one is longer).",
                            shorter
                        ));
                    }
                }

                Some((message, vec![]))
            },
        )
    }
}

impl Snapshotting<Vec<u8>, Vec<u8>> {
    /// Create a raw binary data snapshot strategy.
    ///
    /// The value (`Vec<u8>`) is used directly as the snapshot format. Comparison
    /// is byte-by-byte, reporting the position of the first difference and any
    /// size mismatch. Stored on disk as `.bin` files.
    ///
    /// # Requirements
    ///
    /// - REQ-STRAT-006a: `path_extension` = `"bin"`
    /// - REQ-STRAT-006b: `to_data` / `from_data` = identity
    /// - REQ-STRAT-006c: `diff` = byte equality check with size and position info
    ///
    /// # Examples
    ///
    /// ```
    /// use snapshot_testing::Snapshotting;
    ///
    /// let strategy = Snapshotting::<Vec<u8>, Vec<u8>>::data();
    /// assert_eq!(strategy.path_extension.as_deref(), Some("bin"));
    /// ```
    ///
    /// Round-trip through serialization preserves data:
    ///
    /// ```
    /// use snapshot_testing::Snapshotting;
    ///
    /// let strategy = Snapshotting::<Vec<u8>, Vec<u8>>::data();
    /// let original = vec![0xDE, 0xAD, 0xBE, 0xEF];
    /// let serialized = (strategy.diffing.to_data)(&original);
    /// let deserialized = (strategy.diffing.from_data)(&serialized);
    /// assert_eq!(original, deserialized);
    /// ```
    pub fn data() -> Self {
        Snapshotting::new(
            Some("bin"),
            Diffing::<Vec<u8>>::data(),
            |value: &Vec<u8>| std::future::ready(value.clone()),
        )
    }
}
