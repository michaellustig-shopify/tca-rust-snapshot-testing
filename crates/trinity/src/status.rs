//! ┌─────────────────────────────────────────────────────┐
//! │  status.rs — `trinity status` command               │
//! ├─────────────────────────────────────────────────────┤
//! │  WHAT: Reads .trinity/state.json and displays a     │
//! │  human-readable summary of the workspace state.     │
//! │                                                     │
//! │  WHY: Developers need a quick way to see if Trinity │
//! │  is set up and what the current coverage looks like │
//! │  without running a full check.                      │
//! │                                                     │
//! │  ALTERNATIVES:                                      │
//! │  • cat .trinity/state.json — raw, not friendly      │
//! │  • Dashboard/TUI — overkill for a few numbers       │
//! │                                                     │
//! │  TESTED BY: doc tests, status_tests.rs              │
//! │                                                     │
//! │  EDGE CASES: Not initialized, corrupted state.json, │
//! │  state from a different version.                    │
//! │                                                     │
//! │  CHANGELOG:                                         │
//! │  • v0.1.0 — Initial status display                  │
//! │                                                     │
//! │  HISTORY: git log --oneline --follow -- status.rs   │
//! └─────────────────────────────────────────────────────┘

use std::path::Path;

use crate::error::{TrinityError, TrinityResult};
use crate::state;

/// Runs the `trinity status` command.
///
/// Reads the state file and prints a formatted summary. If Trinity is not
/// initialized, prints a helpful message instead of an error trace.
///
/// # Examples
///
/// ```no_run
/// use trinity::status::run_status;
/// use std::path::Path;
///
/// run_status(Path::new(".")).expect("status failed");
/// ```
pub fn run_status(workspace_root: &Path) -> TrinityResult<()> {
    match state::read_state(workspace_root) {
        Ok(st) => {
            println!();
            println!("┌───────────────────────────────────────┐");
            println!("│  TRINITY STATUS                       │");
            println!("├───────────────────────────────────────┤");
            println!(
                "│  Initialized:    {:<20} │",
                if st.initialized { "YES" } else { "NO" }
            );
            println!(
                "│  Last scan:      {:<20} │",
                st.timestamp.format("%Y-%m-%d %H:%M UTC")
            );
            println!(
                "│  Files tracked:  {:<20} │",
                st.file_count
            );
            println!(
                "│  Undocumented:   {:<20} │",
                st.undocumented_count
            );
            println!(
                "│  Untested fns:   {:<20} │",
                st.untested_count
            );
            println!("└───────────────────────────────────────┘");

            if st.undocumented_count > 0 || st.untested_count > 0 {
                println!();
                println!("Run `trinity check` to see specific items that need attention.");
            }

            Ok(())
        }
        Err(TrinityError::NotInitialized) => {
            println!();
            println!("Trinity is not initialized in this workspace.");
            println!();
            println!("Run `trinity init` to get started.");
            println!("This will scan your codebase and install the pre-commit hook.");
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Formats a `TrinityState` into a status string without printing.
///
/// Useful for testing or embedding in other output.
///
/// # Examples
///
/// ```
/// use trinity::status::format_status;
/// use trinity::state::TrinityState;
///
/// let state = TrinityState::default();
/// let text = format_status(&state);
/// assert!(text.contains("Initialized: NO"));
/// ```
pub fn format_status(st: &state::TrinityState) -> String {
    format!(
        "Initialized: {}\nLast scan: {}\nFiles: {}\nUndocumented: {}\nUntested: {}",
        if st.initialized { "YES" } else { "NO" },
        st.timestamp.format("%Y-%m-%d %H:%M UTC"),
        st.file_count,
        st.undocumented_count,
        st.untested_count,
    )
}
