//! ┌─────────────────────────────────────────────────────┐
//! │  init.rs — `trinity init` command                   │
//! ├─────────────────────────────────────────────────────┤
//! │  WHAT: Scans the workspace, counts undocumented     │
//! │  and untested items, estimates token cost, asks for │
//! │  confirmation, writes state, and installs the git   │
//! │  pre-commit hook.                                   │
//! │                                                     │
//! │  WHY: Before Trinity can enforce sync, it needs a   │
//! │  baseline of the current codebase state. The user   │
//! │  should see the scope before committing to it.      │
//! │                                                     │
//! │  ALTERNATIVES:                                      │
//! │  • Lazy init on first check — loses the explicit    │
//! │    opt-in and cost estimate                         │
//! │  • Config file instead of scan — doesn't capture    │
//! │    real state                                       │
//! │                                                     │
//! │  TESTED BY: doc tests, init_tests.rs                │
//! │                                                     │
//! │  EDGE CASES: Already initialized, no git repo, no   │
//! │  .rs files, very large codebase, user says no.      │
//! │                                                     │
//! │  CHANGELOG:                                         │
//! │  • v0.1.0 — Initial implementation                  │
//! │                                                     │
//! │  HISTORY: git log --oneline --follow -- init.rs     │
//! └─────────────────────────────────────────────────────┘

use std::io::Write;
use std::path::Path;

use chrono::Utc;

use crate::error::{io_err, TrinityError, TrinityResult};
use crate::scanner::{self, has_corresponding_test, ItemKind};
use crate::state::{self, TrinityState};

/// The content of the git pre-commit hook that Trinity installs.
///
/// This is a minimal shell script that runs `trinity check`. If trinity
/// is not on PATH, it falls back to `cargo run --bin trinity -- check`.
///
/// # Examples
///
/// ```
/// use trinity::init::PRE_COMMIT_HOOK;
///
/// assert!(PRE_COMMIT_HOOK.contains("trinity check"));
/// assert!(PRE_COMMIT_HOOK.starts_with("#!/"));
/// ```
pub const PRE_COMMIT_HOOK: &str = r#"#!/usr/bin/env bash
# Trinity pre-commit hook — ensures docs/tests/code stay in sync.
# Installed by `trinity init`. Do not edit manually.

set -e

# Try trinity on PATH first, fall back to cargo run
if command -v trinity &> /dev/null; then
    trinity check
else
    cargo run --bin trinity -- check
fi
"#;

/// Runs the `trinity init` command.
///
/// Steps:
/// 1. Scan the workspace for all .rs files
/// 2. Parse each file, count pub items, check for docs and tests
/// 3. Estimate token usage
/// 4. Print summary and ask for confirmation
/// 5. Write `.trinity/state.json`
/// 6. Install git pre-commit hook
///
/// # Examples
///
/// ```no_run
/// use trinity::init::run_init;
/// use std::path::Path;
///
/// run_init(Path::new(".")).expect("init failed");
/// ```
pub fn run_init(workspace_root: &Path) -> TrinityResult<()> {
    println!("Scanning workspace...");
    println!();

    // Step 1: Scan
    let scan = scanner::scan_workspace(workspace_root)?;

    // Step 2: Count untested functions
    let mut untested_count = 0;
    let pub_fns: Vec<_> = scan
        .items
        .iter()
        .filter(|i| i.kind == ItemKind::Function)
        .collect();

    // Collect test names from all scanned files
    let mut all_test_names = Vec::new();
    let rs_files = scanner::find_rs_files(workspace_root)?;
    for file in &rs_files {
        if let Ok(source) = std::fs::read_to_string(file) {
            let names = scanner::find_test_names_in_source(&source);
            all_test_names.extend(names);
        }
    }

    for func in &pub_fns {
        if !has_corresponding_test(&func.name, &all_test_names) {
            untested_count += 1;
        }
    }

    let undocumented = scan.undocumented_count();
    let tokens = scan.estimated_tokens();

    // Step 3: Print summary
    println!("  Files scanned:     {}", scan.file_count);
    println!("  Total lines:       {}", scan.total_lines);
    println!("  Public items:      {}", scan.items.len());
    println!("  Undocumented:      {undocumented}");
    println!("  Untested (fns):    {untested_count}");
    println!("  Parse errors:      {}", scan.parse_errors.len());
    println!();
    println!(
        "  Estimated token usage (3 passes): ~{tokens} tokens"
    );
    println!();

    if !scan.parse_errors.is_empty() {
        println!("  Files with parse errors:");
        for (path, msg) in &scan.parse_errors {
            println!("    {} -- {msg}", path.display());
        }
        println!();
    }

    // Step 4: Ask for confirmation
    print!("Proceed with initialization? [y/N] ");
    std::io::stdout()
        .flush()
        .map_err(|e| io_err(workspace_root.to_path_buf(), e))?;

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| io_err(workspace_root.to_path_buf(), e))?;

    let answer = input.trim().to_lowercase();
    if answer != "y" && answer != "yes" {
        return Err(TrinityError::Cancelled);
    }

    // Step 5: Write state
    let st = TrinityState {
        initialized: true,
        timestamp: Utc::now(),
        file_count: scan.file_count,
        undocumented_count: undocumented,
        untested_count,
    };
    state::write_state(workspace_root, &st)?;
    println!("  Wrote .trinity/state.json");

    // Step 6: Install git hook
    install_pre_commit_hook(workspace_root)?;

    println!();
    println!("Trinity initialized successfully.");
    println!("The pre-commit hook will run `trinity check` on every commit.");
    Ok(())
}

/// Installs the Trinity pre-commit hook into `.git/hooks/pre-commit`.
///
/// If a pre-commit hook already exists, it is backed up to
/// `.git/hooks/pre-commit.bak` before being overwritten.
///
/// Returns an error if no `.git` directory is found (not a git repo).
///
/// # Examples
///
/// ```no_run
/// use trinity::init::install_pre_commit_hook;
/// use std::path::Path;
///
/// install_pre_commit_hook(Path::new(".")).expect("hook install failed");
/// ```
pub fn install_pre_commit_hook(workspace_root: &Path) -> TrinityResult<()> {
    let git_dir = workspace_root.join(".git");
    if !git_dir.is_dir() {
        return Err(TrinityError::Git(
            "No .git directory found. Is this a git repository?".to_string(),
        ));
    }

    let hooks_dir = git_dir.join("hooks");
    std::fs::create_dir_all(&hooks_dir)
        .map_err(|e| io_err(hooks_dir.clone(), e))?;

    let hook_path = hooks_dir.join("pre-commit");

    // Back up existing hook
    if hook_path.exists() {
        let backup = hooks_dir.join("pre-commit.bak");
        std::fs::copy(&hook_path, &backup)
            .map_err(|e| io_err(hook_path.clone(), e))?;
        println!("  Backed up existing pre-commit hook to pre-commit.bak");
    }

    // Write new hook
    std::fs::write(&hook_path, PRE_COMMIT_HOOK)
        .map_err(|e| io_err(hook_path.clone(), e))?;

    // Make executable (Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&hook_path, perms)
            .map_err(|e| io_err(hook_path.clone(), e))?;
    }

    println!("  Installed pre-commit hook at .git/hooks/pre-commit");
    Ok(())
}
