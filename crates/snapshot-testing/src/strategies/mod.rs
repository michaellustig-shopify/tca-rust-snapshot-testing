//! +---------------------------------------------------------+
//! |  STRATEGIES                                             |
//! |  Built-in snapshot strategies for common types          |
//! +---------------------------------------------------------+
//! |                                                         |
//! |  lines ──────── String as text lines (foundation)       |
//! |  json ───────── T: Serialize as pretty JSON             |
//! |  debug ──────── T: Debug as {:#?} output                |
//! |  description ── T: Display as to_string() output        |
//! |  data ───────── Vec<u8> as raw binary bytes             |
//! |                                                         |
//! +---------------------------------------------------------+
//! |  WHY: These are the bread-and-butter strategies that    |
//! |  cover the vast majority of snapshot testing use cases.  |
//! |  Each strategy is defined as an associated function on   |
//! |  `Snapshotting` (not a standalone function), mirroring  |
//! |  the Swift API: `Snapshotting.lines`, `.json`, etc.     |
//! |                                                         |
//! |  SWIFT EQUIVALENT: Sources/SnapshotTesting/Snapshotting/ |
//! |  (multiple files, one per strategy)                     |
//! |                                                         |
//! |  ARCHITECTURE: Each strategy module adds an `impl`      |
//! |  block on `Snapshotting` (and sometimes `Diffing`).     |
//! |  This is Rust's equivalent of Swift extensions -- the   |
//! |  type is defined in `snapshotting.rs` and extended here. |
//! |                                                         |
//! |  CHANGELOG:                                             |
//! |  * v0.1.0 -- lines, json, debug, data, description     |
//! +---------------------------------------------------------+

pub mod data;
pub mod debug;
pub mod description;
pub mod json;
pub mod lines;
