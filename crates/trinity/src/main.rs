//! ┌─────────────────────────────────────────────────────┐
//! │  TRINITY                                            │
//! │  Pre-commit sync enforcement for docs/tests/code    │
//! ├─────────────────────────────────────────────────────┤
//! │                                                     │
//! │           ┌─────────┐                               │
//! │      ┌────│  DOCS   │────┐                          │
//! │      │    └─────────┘    │                          │
//! │      │                   │                          │
//! │  ┌───▼───┐          ┌───▼───┐                      │
//! │  │ TESTS │◄────────►│ CODE  │                      │
//! │  └───────┘          └───────┘                      │
//! │                                                     │
//! │  Three sources of truth, always in sync.            │
//! │  If any pair diverges, the commit is rejected.      │
//! │                                                     │
//! ├─────────────────────────────────────────────────────┤
//! │  WHY: Documentation, tests, and source code drift   │
//! │  apart over time. Trinity makes drift impossible    │
//! │  by checking synchronization at every commit.       │
//! │                                                     │
//! │  COMMANDS:                                          │
//! │  - trinity init   — Scan codebase, install hook     │
//! │  - trinity check  — Verify sync (pre-commit hook)   │
//! │  - trinity status — Show current sync state         │
//! │                                                     │
//! │  TESTED BY: doc tests across all modules            │
//! │  EDGE CASES: uninitialized state, empty repos,      │
//! │  binary files, generated code                       │
//! │                                                     │
//! │  CHANGELOG:                                         │
//! │  - v0.1.0 — Initial CLI scaffold with clap          │
//! │  - v0.2.0 — Full implementation of init/check/      │
//! │             status with syn parsing                  │
//! │                                                     │
//! │  HISTORY: git log --oneline --follow -- main.rs     │
//! └─────────────────────────────────────────────────────┘

use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod check;
mod error;
mod init;
mod scanner;
mod state;
mod status;

/// Trinity — Pre-commit sync enforcement.
///
/// Ensures three sources of truth stay synchronized:
/// 1. Documentation (doc comments + SRS + file headers)
/// 2. Tests (unit tests + doc tests + integration tests)
/// 3. Source code (the actual implementation)
///
/// # Examples
///
/// ```text
/// trinity init     # Scan codebase, install pre-commit hook
/// trinity check    # Verify docs/tests/code are in sync
/// trinity status   # Show current Trinity state
/// ```
#[derive(Parser, Debug)]
#[command(
    name = "trinity",
    version,
    about = "Pre-commit sync enforcement for docs/tests/code"
)]
struct Cli {
    /// Path to the workspace root. Defaults to current directory.
    #[arg(short, long, default_value = ".")]
    workspace: PathBuf,

    #[command(subcommand)]
    command: Command,
}

/// Available Trinity subcommands.
///
/// Each command corresponds to a phase of the Trinity workflow:
/// - `init`: One-time setup that scans the codebase and installs the hook
/// - `check`: Called by the pre-commit hook on every commit
/// - `status`: Human-readable summary of sync state
#[derive(Subcommand, Debug)]
enum Command {
    /// Initialize Trinity: scan codebase, write state, install git hook.
    ///
    /// Walks the workspace, counts undocumented and untested items, shows
    /// a summary with token cost estimate, asks for confirmation, then
    /// writes .trinity/state.json and installs the pre-commit hook.
    Init,

    /// Run the three-way sync check on staged changes.
    ///
    /// Reads `git diff --cached` and checks every staged .rs file:
    /// 1. Docs: every pub item has a doc comment (with code blocks for fns)
    /// 2. Tests: every pub fn has a corresponding test
    /// 3. SRS: every pub item appears in artifacts/latest/SRS.md
    ///
    /// Exits 0 if all pass, 1 if any fail.
    Check,

    /// Show current Trinity sync status.
    ///
    /// Reads .trinity/state.json and displays whether Trinity is
    /// initialized, when the last scan was, and coverage counts.
    Status,
}

/// Entry point for the Trinity CLI.
///
/// Parses arguments with clap, dispatches to the appropriate command
/// handler, and translates errors into process exit codes.
fn main() {
    let cli = Cli::parse();
    let workspace = &cli.workspace;

    let result = match cli.command {
        Command::Init => init::run_init(workspace).map(|()| 0),
        Command::Check => check::run_check(workspace),
        Command::Status => status::run_status(workspace).map(|()| 0),
    };

    match result {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
