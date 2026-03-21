//! ┌─────────────────────────────────────────────────────┐
//! │  TRINITY                                             │
//! │  Pre-commit sync enforcement for docs/tests/code    │
//! ├─────────────────────────────────────────────────────┤
//! │                                                      │
//! │           ┌─────────┐                                │
//! │      ┌────│  DOCS   │────┐                           │
//! │      │    └─────────┘    │                           │
//! │      │                   │                           │
//! │  ┌───▼───┐          ┌───▼───┐                       │
//! │  │ TESTS │◄────────►│ CODE  │                       │
//! │  └───────┘          └───────┘                       │
//! │                                                      │
//! │  Three sources of truth, always in sync.             │
//! │  If any pair diverges, the commit is rejected.       │
//! │                                                      │
//! ├─────────────────────────────────────────────────────┤
//! │  WHY: Documentation, tests, and source code drift    │
//! │  apart over time. Trinity makes drift impossible     │
//! │  by checking synchronization at every commit.        │
//! │                                                      │
//! │  COMMANDS:                                           │
//! │  • trinity init   — Scan codebase, generate SRS,     │
//! │                     add doc comments, install hook   │
//! │  • trinity check  — Verify sync (pre-commit hook)    │
//! │  • trinity status — Show current sync state          │
//! │                                                      │
//! │  TESTED BY: tests/trinity_tests.rs                   │
//! │  EDGE CASES: uninitialized state, empty repos,       │
//! │  binary files, generated code                        │
//! │                                                      │
//! │  CHANGELOG:                                          │
//! │  • v0.1.0 — Initial CLI scaffold with clap           │
//! │                                                      │
//! │  HISTORY: git log --oneline --follow -- crates/trinity/
//! └─────────────────────────────────────────────────────┘

use clap::{Parser, Subcommand};

/// Trinity — Pre-commit sync enforcement.
///
/// Ensures three sources of truth stay synchronized:
/// 1. Documentation (doc comments + SRS + file headers)
/// 2. Tests (unit tests + doc tests + integration tests)
/// 3. Source code (the actual implementation)
///
/// # Examples
///
/// Initialize Trinity for a new project:
/// ```text
/// trinity init
/// ```
///
/// Manually run the sync check:
/// ```text
/// trinity check
/// ```
///
/// View current sync status:
/// ```text
/// trinity status
/// ```
#[derive(Parser, Debug)]
#[command(name = "trinity", version, about = "Pre-commit sync enforcement for docs/tests/code")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// Available Trinity subcommands.
///
/// Each command corresponds to a phase of the Trinity workflow:
/// - `init`: One-time setup that scans the codebase
/// - `check`: Called by the pre-commit hook on every commit
/// - `status`: Human-readable summary of sync state
#[derive(Subcommand, Debug)]
enum Command {
    /// Initialize Trinity: scan codebase, generate SRS, add doc comments, install git hook.
    ///
    /// WARNING: This will scan your entire codebase and invoke Claude agents.
    /// Estimated token usage depends on codebase size (~4 tokens/line × 3 passes).
    Init,

    /// Run the three-way sync check on staged changes.
    ///
    /// Reads `git diff --cached` and fires 3 parallel verification agents:
    /// 1. Docs ↔ Code: doc comments and file headers match implementation
    /// 2. Tests ↔ Code: test assertions match actual behavior
    /// 3. SRS ↔ Code: public API documented in SRS
    ///
    /// Also verifies that all doc comments contain runnable doc tests
    /// (`cargo test --doc` must pass for all changed files).
    Check,

    /// Show current Trinity sync status.
    ///
    /// Displays:
    /// - Whether Trinity is initialized
    /// - Last check timestamp and result
    /// - Count of undocumented items
    /// - Count of untested functions
    /// - SRS coverage percentage
    Status,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Init => {
            println!("⚠ Trinity will scan your entire codebase and invoke Claude agents.");
            println!("  Estimated token usage: calculating...");
            println!();

            // TODO: Count files, estimate tokens, prompt user
            // TODO: Scan for undocumented items
            // TODO: Fire sub-agents to add doc comments (with runnable doc tests)
            // TODO: Generate SRS
            // TODO: Install git pre-commit hook
            // TODO: Write .trinity/state.json

            println!("Trinity initialization not yet implemented.");
        }
        Command::Check => {
            // TODO: Read git diff --cached
            // TODO: Fire 3 parallel agents
            // TODO: Verify doc tests pass (cargo test --doc)
            // TODO: Report results, exit 1 on failure

            println!("Trinity check not yet implemented.");
        }
        Command::Status => {
            // TODO: Read .trinity/state.json
            // TODO: Display current state

            println!("Trinity status not yet implemented.");
        }
    }
}
