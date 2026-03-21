//! ┌─────────────────────────────────────────────────────┐
//! │  DIFF ALGORITHM                                      │
//! │  Line-based LCS (Longest Common Subsequence) diff    │
//! ├─────────────────────────────────────────────────────┤
//! │                                                      │
//! │   "hello\nworld"    "hello\nearth"                   │
//! │         │                  │                          │
//! │         └──────┬───────────┘                          │
//! │                ▼                                      │
//! │          LCS algorithm                                │
//! │                │                                      │
//! │                ▼                                      │
//! │   @@ -1,2 +1,2 @@                                    │
//! │    hello                                              │
//! │   -world                                              │
//! │   +earth                                              │
//! │                                                      │
//! ├─────────────────────────────────────────────────────┤
//! │  WHY: Text diffing is the foundation for most        │
//! │  snapshot comparisons. JSON, dumps, descriptions     │
//! │  all diff as text lines.                             │
//! │                                                      │
//! │  ALTERNATIVES: `similar` crate (used internally      │
//! │  for the actual LCS). Myers diff (similar uses it).  │
//! │  We wrap similar to produce our specific hunk format.│
//! │                                                      │
//! │  SWIFT EQUIVALENT: Diff.swift                        │
//! │  (Sources/SnapshotTesting/Diff.swift)                │
//! │                                                      │
//! │  TESTED BY: tests/diff_tests.rs                      │
//! │  EDGE CASES: empty strings, single lines, no diff,   │
//! │  all different, unicode, trailing newlines            │
//! │                                                      │
//! │  CHANGELOG:                                          │
//! │  • v0.1.0 — Initial port using `similar` crate       │
//! │                                                      │
//! │  HISTORY: git log --oneline --follow -- crates/snapshot-testing/src/diff.rs
//! └─────────────────────────────────────────────────────┘

use similar::{ChangeTag, TextDiff};

/// Produce a unified diff between two strings.
///
/// Returns `None` if the strings are identical, or `Some(diff_string)`
/// with a unified diff format (similar to `diff -u`).
///
/// # Arguments
///
/// - `old`: The reference/expected text
/// - `new`: The actual/snapshot text
/// - `context_lines`: Number of unchanged lines to show around changes (default: 3)
///
/// # Swift Equivalent
///
/// This replaces the LCS-based diff algorithm in `Diff.swift` which
/// produces hunks with `@@` markers. We use the `similar` crate's
/// implementation of Myers' diff algorithm, which is the same algorithm
/// used by `git diff`.
///
/// # Why `similar` instead of hand-rolling?
///
/// The Swift version implements LCS from scratch (~200 lines). In Rust,
/// `similar` provides a battle-tested Myers diff with O(ND) complexity.
/// No reason to reimplement when a well-maintained crate exists.
pub fn line_diff(old: &str, new: &str, context_lines: usize) -> Option<String> {
    if old == new {
        return None;
    }

    let diff = TextDiff::from_lines(old, new);
    let mut output = String::new();

    for hunk in diff.unified_diff().context_radius(context_lines).iter_hunks() {
        output.push_str(&format!("{hunk}"));
    }

    if output.is_empty() {
        None
    } else {
        Some(output)
    }
}

/// Produce a character-level diff for single-line comparisons.
///
/// Returns `None` if strings are identical, `Some(diff)` with
/// inline markers showing exactly which characters differ.
///
/// Useful for comparing short strings where line-level diff
/// would just show "this line changed" without indicating where.
pub fn inline_diff(old: &str, new: &str) -> Option<String> {
    if old == new {
        return None;
    }

    let diff = TextDiff::from_chars(old, new);
    let mut expected = String::from("Expected: ");
    let mut actual = String::from("Actual:   ");

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Equal => {
                expected.push_str(change.as_str().unwrap_or(""));
                actual.push_str(change.as_str().unwrap_or(""));
            }
            ChangeTag::Delete => {
                expected.push_str(&format!("[{}]", change.as_str().unwrap_or("")));
            }
            ChangeTag::Insert => {
                actual.push_str(&format!("[{}]", change.as_str().unwrap_or("")));
            }
        }
    }

    Some(format!("{expected}\n{actual}"))
}
