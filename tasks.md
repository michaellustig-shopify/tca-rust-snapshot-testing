# Tasks

## In Progress
- [ ] Port Swift test files to Rust `#[test]` functions (16 test files, ~2672 lines)
- [ ] Generate SRS artifacts for v1.0.0

## Up Next
- [ ] Implement `Snapshotting::lines()` — text line-based strategy
- [ ] Implement `Snapshotting::json()` — JSON serialization strategy
- [ ] Implement `Snapshotting::dump()` — custom dump strategy
- [ ] Implement snapshot file counter (for multiple snapshots per test)
- [ ] Implement inline snapshot assertion macro
- [ ] Build Trinity `init` — codebase scanning and doc generation
- [ ] Build Trinity `check` — 3-agent pre-commit verification
- [ ] Build Trinity `status` — sync state display
- [ ] Add doc tests to all public functions
- [ ] Create teaching curriculum (docs/curriculum/)
- [ ] Wire pre-commit hook installation

## Done
- [x] Initialize Rust workspace — Saturday, March 21st ~3:30 PM
- [x] Create core type stubs (Snapshotting, Diffing, Record, DiffTool) — Saturday, March 21st ~3:30 PM
- [x] Implement line_diff and inline_diff — Saturday, March 21st ~3:30 PM
- [x] Implement verify_snapshot / assert_snapshot — Saturday, March 21st ~3:30 PM
- [x] Trinity CLI scaffold — Saturday, March 21st ~3:30 PM
- [x] Create port-to-rust skill — Saturday, March 21st ~3:25 PM
