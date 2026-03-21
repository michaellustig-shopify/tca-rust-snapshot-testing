//! ┌─────────────────────────────────────────────────────┐
//! │  state.rs — Trinity persistent state                │
//! ├─────────────────────────────────────────────────────┤
//! │  WHAT: Reads and writes .trinity/state.json which   │
//! │  records whether trinity has been initialized and   │
//! │  the last scan results.                             │
//! │                                                     │
//! │  WHY: Trinity needs to know if init has been run    │
//! │  before check can execute. The state file lives in  │
//! │  the workspace root so it can be git-ignored.       │
//! │                                                     │
//! │  ALTERNATIVES:                                      │
//! │  • Database (SQLite) — overkill for a few fields    │
//! │  • TOML/YAML — JSON is already a dependency        │
//! │                                                     │
//! │  TESTED BY: doc tests below, integration tests      │
//! │                                                     │
//! │  EDGE CASES: Missing .trinity dir, corrupted JSON,  │
//! │  concurrent writes, permission errors.              │
//! │                                                     │
//! │  CHANGELOG:                                         │
//! │  • v0.1.0 — Initial state struct and IO             │
//! │                                                     │
//! │  HISTORY: git log --oneline --follow -- state.rs    │
//! └─────────────────────────────────────────────────────┘

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{io_err, TrinityError, TrinityResult};

/// The directory name where Trinity stores its metadata.
const TRINITY_DIR: &str = ".trinity";

/// The filename for the state file inside the Trinity directory.
const STATE_FILE: &str = "state.json";

/// Persistent state for a Trinity-managed workspace.
///
/// Serialized as JSON to `.trinity/state.json` at the workspace root.
///
/// # Examples
///
/// ```
/// use trinity::state::TrinityState;
///
/// let state = TrinityState::default();
/// assert!(!state.initialized);
/// assert_eq!(state.file_count, 0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrinityState {
    /// Whether `trinity init` has been run successfully.
    pub initialized: bool,

    /// When the last init or check was run.
    pub timestamp: DateTime<Utc>,

    /// Number of .rs files found in the workspace.
    pub file_count: usize,

    /// Number of public items missing doc comments.
    pub undocumented_count: usize,

    /// Number of public functions missing corresponding tests.
    pub untested_count: usize,
}

impl Default for TrinityState {
    /// Creates a default uninitialized state.
    ///
    /// # Examples
    ///
    /// ```
    /// use trinity::state::TrinityState;
    ///
    /// let s = TrinityState::default();
    /// assert!(!s.initialized);
    /// ```
    fn default() -> Self {
        Self {
            initialized: false,
            timestamp: Utc::now(),
            file_count: 0,
            undocumented_count: 0,
            untested_count: 0,
        }
    }
}

/// Returns the path to the `.trinity` directory for a given workspace root.
///
/// # Examples
///
/// ```
/// use trinity::state::trinity_dir;
/// use std::path::Path;
///
/// let dir = trinity_dir(Path::new("/some/project"));
/// assert_eq!(dir.to_str().unwrap(), "/some/project/.trinity");
/// ```
pub fn trinity_dir(workspace_root: &Path) -> PathBuf {
    workspace_root.join(TRINITY_DIR)
}

/// Returns the path to `.trinity/state.json` for a given workspace root.
///
/// # Examples
///
/// ```
/// use trinity::state::state_file_path;
/// use std::path::Path;
///
/// let p = state_file_path(Path::new("/some/project"));
/// assert!(p.to_str().unwrap().ends_with("state.json"));
/// ```
pub fn state_file_path(workspace_root: &Path) -> PathBuf {
    trinity_dir(workspace_root).join(STATE_FILE)
}

/// Reads the Trinity state from disk.
///
/// Returns `TrinityError::NotInitialized` if the state file does not exist.
/// Returns `TrinityError::Json` if the file contents are not valid JSON.
///
/// # Examples
///
/// ```no_run
/// use trinity::state::read_state;
/// use std::path::Path;
///
/// let state = read_state(Path::new(".")).expect("state read failed");
/// println!("initialized: {}", state.initialized);
/// ```
pub fn read_state(workspace_root: &Path) -> TrinityResult<TrinityState> {
    let path = state_file_path(workspace_root);
    if !path.exists() {
        return Err(TrinityError::NotInitialized);
    }
    let contents =
        std::fs::read_to_string(&path).map_err(|e| io_err(path.clone(), e))?;
    let state: TrinityState = serde_json::from_str(&contents)?;
    Ok(state)
}

/// Writes the Trinity state to disk, creating the `.trinity` directory if needed.
///
/// # Examples
///
/// ```no_run
/// use trinity::state::{write_state, TrinityState};
/// use std::path::Path;
///
/// let state = TrinityState::default();
/// write_state(Path::new("/tmp/test-project"), &state).expect("write failed");
/// ```
pub fn write_state(workspace_root: &Path, state: &TrinityState) -> TrinityResult<()> {
    let dir = trinity_dir(workspace_root);
    std::fs::create_dir_all(&dir).map_err(|e| io_err(dir.clone(), e))?;

    let path = state_file_path(workspace_root);
    let json = serde_json::to_string_pretty(state)?;
    std::fs::write(&path, json).map_err(|e| io_err(path, e))?;
    Ok(())
}

/// Checks whether Trinity has been initialized in the given workspace.
///
/// This is a quick file-existence check — it does not validate JSON contents.
///
/// # Examples
///
/// ```
/// use trinity::state::is_initialized;
/// use std::path::Path;
///
/// // A random temp dir won't have .trinity/state.json
/// let tmp = std::env::temp_dir().join("trinity-test-not-exist");
/// assert!(!is_initialized(&tmp));
/// ```
pub fn is_initialized(workspace_root: &Path) -> bool {
    state_file_path(workspace_root).exists()
}
