# Progress

## Saturday, March 21st at ~3:30 PM EDT
- Initialized Rust workspace with 4 crates: `snapshot-testing`, `inline-snapshot-testing`, `snapshot-testing-custom-dump`, `trinity`
- Created core type stubs: `Snapshotting<V,F>`, `Diffing<V>`, `Record`, `DiffTool`, `SnapshotTestingConfiguration`
- Implemented `line_diff` and `inline_diff` using the `similar` crate
- Implemented `verify_snapshot` and `assert_snapshot` with file I/O and record modes
- Implemented `with_snapshot_testing` thread-local configuration scoping
- Trinity CLI scaffold with `init`, `check`, `status` subcommands
- All crates compile, 2 doc tests passing
- Rich ASCII art file headers on all `.rs` files
- Commit: _(pending initial commit)_
