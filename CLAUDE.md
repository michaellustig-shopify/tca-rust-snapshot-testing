# rust-snapshot-testing

A Rust port of [Point-Free's swift-snapshot-testing](https://github.com/pointfreeco/swift-snapshot-testing).

## Source Package

- Upstream repo: <https://github.com/pointfreeco/swift-snapshot-testing>
- Local checkout: `/Users/michael.lustig/Sync/Sigil/.build/checkouts/swift-snapshot-testing`
- Authors: Brandon Williams & Stephen Celis (Point-Free)

## Rules

- Always keep `artifacts/<version>/SRS.md` in sync with source code changes and tests
- Run `cargo test` and `cargo clippy` before every commit
- **Trinity must be initialized** — commits fail without it (`trinity init`)
- Port tests FIRST, then implement to make tests pass
- Every function, struct, enum, trait, and module gets `///` doc comments. No exceptions.
- If a file is too big to document thoroughly, break it into smaller modules or crates
- Use the `insta` crate for snapshot testing assertions
- Prefer `&str` over `String` in function parameters
- Derive `Debug, Clone, PartialEq` on all public types
- Use `thiserror` for error types, `#[must_use]` on important return values
- Every `.rs` file gets a rich ASCII art header (see port-to-rust skill, File Headers section)
- Update `progress.md` with human-readable timestamps and GitHub commit links after each commit

## Trinity — Pre-Commit Sync Enforcement

Trinity is a Rust binary in the `crates/trinity/` workspace member. It ensures three sources of truth stay in sync:

1. **Documentation** — `///` doc comments + file headers accurately describe the code
2. **Tests** — Every code path has corresponding tests that match behavior
3. **Source code** — Implementation matches what docs say and tests assert

On every commit, Trinity fires 3 parallel Claude agents checking docs↔code, tests↔code, and SRS↔code. Commit is rejected if any mismatch is found.

### Trinity Commands

- `trinity init` — Scan codebase, generate SRS, add docs, install pre-commit hook (warns about token usage)
- `trinity check` — Run the 3-agent verification on staged changes
- `trinity status` — Show current sync state

## Versioning & Artifacts

```
artifacts/
├── v1.0.0/
│   ├── SRS.md              # Software Requirement Specification
│   ├── architecture.md     # Module diagrams, dependency graphs
│   ├── type-mapping.md     # Swift → Rust type mapping
│   └── verification.md     # Requirement → test matrix
└── latest -> v1.0.0
```

Version everything consistently: artifacts, SRS, file header changelogs, and upstream version all align.

## Module Mapping (Swift → Rust)

| Swift Target | Rust Crate | Purpose |
|---|---|---|
| SnapshotTesting | snapshot-testing | Core diffing & assertion engine |
| InlineSnapshotTesting | inline-snapshot-testing | Source-embedded inline snapshots |
| SnapshotTestingCustomDump | snapshot-testing-custom-dump | Pretty-print integration |
| _(new)_ | trinity | Pre-commit sync enforcement tool |

## Key Type Mapping

| Swift | Rust | Notes |
|---|---|---|
| `Snapshotting<Value, Format>` | `Snapshotting<V, F>` | Core strategy trait/struct |
| `Diffing<Value>` | `Diffing<V>` | Comparison + serialization |
| `Async<Value>` | `impl Future<Output = V>` | Use std futures |
| `Record` enum | `Record` enum | `.all`, `.failed`, `.missing`, `.never` |
| `DiffTool` | `DiffTool` | Custom diff command |

## How to Build & Test

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo run --bin trinity -- init    # Initialize Trinity
cargo run --bin trinity -- check   # Manual sync check
```

## How to Update When Upstream Releases

1. Check upstream tags: `git -C <upstream-clone> tag --sort=-v:refname | head`
2. Create new `artifacts/<new-version>/` directory
3. Run SRS generation agents for the new version
4. Update `delta.md` with what changed
5. Port new/changed tests
6. Implement changes
7. Run Trinity to verify everything is in sync
