# Tasks

## In Progress
- [ ] Create teaching curriculum (docs/curriculum/) — agent working
- [ ] Un-ignore ported tests by implementing remaining types

## Up Next
- [ ] Implement snapshot file counter (for multiple snapshots per test)
- [ ] Implement inline snapshot assertion macro (source rewriting with syn)
- [ ] Implement custom dump strategy (pretty-print integration)
- [ ] Implement `Snapshotting::func()` — closure-based custom strategy
- [ ] Implement `Snapshotting::wait()` — delayed async strategy
- [ ] Implement path sanitization for snapshot file names
- [ ] Add panic-safe config guard (drop guard for with_snapshot_testing)
- [ ] Wire Trinity agent calls (currently uses syn-based static analysis)
- [ ] Push to GitHub and add remote
- [ ] Neo4j knowledge ingestion

## Done
- [x] Create port-to-rust skill — ~3:25 PM
- [x] Initialize Rust workspace — ~3:30 PM
- [x] Create core types (Snapshotting, Diffing, Record, DiffTool) — ~3:30 PM
- [x] Implement line_diff and inline_diff — ~3:30 PM
- [x] Implement verify_snapshot / assert_snapshot — ~3:30 PM
- [x] Port 94 tests from Swift (62 core + 32 inline) — ~4:00 PM
- [x] Generate SRS artifacts (2,500 lines, 75 requirements) — ~4:00 PM
- [x] Build Trinity (init/check/status with syn parsing, 39 doc tests) — ~4:20 PM
- [x] Implement 5 strategies (lines, json, debug, data, description) — ~4:35 PM
