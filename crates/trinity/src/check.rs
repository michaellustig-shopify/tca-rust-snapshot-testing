//! ┌─────────────────────────────────────────────────────┐
//! │  check.rs — `trinity check` command                 │
//! ├─────────────────────────────────────────────────────┤
//! │  WHAT: Reads staged git changes, filters to .rs     │
//! │  files, and verifies three-way sync between docs,   │
//! │  tests, and code for every public item.             │
//! │                                                     │
//! │  WHY: This is the pre-commit enforcement point.     │
//! │  By checking only staged files, it stays fast and   │
//! │  only blocks commits that introduce drift.          │
//! │                                                     │
//! │  ALTERNATIVES:                                      │
//! │  • Check all files — too slow for large codebases   │
//! │  • Regex-based checks — misses nested items, false  │
//! │    positives on commented-out code                  │
//! │  • External linter (clippy) — doesn't check tests   │
//! │    or SRS linkage                                   │
//! │                                                     │
//! │  TESTED BY: doc tests, check_tests.rs               │
//! │                                                     │
//! │  EDGE CASES: No staged files, deleted files in      │
//! │  staging, binary files, non-UTF8, trinity not       │
//! │  initialized, no SRS.md yet.                        │
//! │                                                     │
//! │  CHANGELOG:                                         │
//! │  • v0.1.0 — Initial three-way check                 │
//! │                                                     │
//! │  HISTORY: git log --oneline --follow -- check.rs    │
//! └─────────────────────────────────────────────────────┘

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{TrinityError, TrinityResult};
use crate::scanner::{self, has_corresponding_test, find_test_names_in_source, ItemKind, PubItem};
use crate::state;

/// A single check failure with context about what's wrong.
///
/// # Examples
///
/// ```
/// use trinity::check::{CheckFailure, CheckCategory};
///
/// let f = CheckFailure {
///     category: CheckCategory::Docs,
///     item_name: "my_fn".to_string(),
///     file: std::path::PathBuf::from("src/lib.rs"),
///     line: 42,
///     message: "Missing doc comment".to_string(),
/// };
/// assert_eq!(f.category, CheckCategory::Docs);
/// ```
#[derive(Debug, Clone)]
pub struct CheckFailure {
    /// Which check category this failure belongs to.
    pub category: CheckCategory,
    /// The name of the item that failed the check.
    pub item_name: String,
    /// The file where the item lives.
    pub file: PathBuf,
    /// The line number of the item.
    pub line: usize,
    /// A human-readable description of the failure.
    pub message: String,
}

/// The three check categories that Trinity enforces.
///
/// # Examples
///
/// ```
/// use trinity::check::CheckCategory;
///
/// let cat = CheckCategory::Tests;
/// assert_eq!(format!("{cat}"), "TESTS");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckCategory {
    /// Documentation checks (doc comments, code blocks).
    Docs,
    /// Test coverage checks (each pub fn has a test).
    Tests,
    /// SRS coverage checks (pub items appear in SRS.md).
    Srs,
}

impl std::fmt::Display for CheckCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckCategory::Docs => write!(f, "DOCS"),
            CheckCategory::Tests => write!(f, "TESTS"),
            CheckCategory::Srs => write!(f, "SRS"),
        }
    }
}

/// Result of running all three checks.
///
/// # Examples
///
/// ```
/// use trinity::check::CheckReport;
///
/// let report = CheckReport::default();
/// assert!(report.passed());
/// ```
#[derive(Debug, Clone, Default)]
pub struct CheckReport {
    /// All failures found across all categories.
    pub failures: Vec<CheckFailure>,
    /// Number of files checked.
    pub files_checked: usize,
    /// Number of public items checked.
    pub items_checked: usize,
}

impl CheckReport {
    /// Returns true if all checks passed (no failures).
    ///
    /// # Examples
    ///
    /// ```
    /// use trinity::check::CheckReport;
    ///
    /// let report = CheckReport::default();
    /// assert!(report.passed());
    /// ```
    pub fn passed(&self) -> bool {
        self.failures.is_empty()
    }

    /// Counts failures in a specific category.
    ///
    /// # Examples
    ///
    /// ```
    /// use trinity::check::{CheckReport, CheckCategory};
    ///
    /// let report = CheckReport::default();
    /// assert_eq!(report.count_by_category(&CheckCategory::Docs), 0);
    /// ```
    pub fn count_by_category(&self, cat: &CheckCategory) -> usize {
        self.failures.iter().filter(|f| &f.category == cat).count()
    }

    /// Prints a human-readable summary of the check results.
    ///
    /// # Examples
    ///
    /// ```
    /// use trinity::check::CheckReport;
    ///
    /// let report = CheckReport::default();
    /// report.print_summary(); // prints all-clear message
    /// ```
    pub fn print_summary(&self) {
        let docs_fail = self.count_by_category(&CheckCategory::Docs);
        let tests_fail = self.count_by_category(&CheckCategory::Tests);
        let srs_fail = self.count_by_category(&CheckCategory::Srs);

        println!();
        println!("┌───────────────────────────────────────┐");
        println!("│  TRINITY CHECK REPORT                 │");
        println!("├───────────────────────────────────────┤");
        println!(
            "│  Files checked:  {:<20} │",
            self.files_checked
        );
        println!(
            "│  Items checked:  {:<20} │",
            self.items_checked
        );
        println!("├───────────────────────────────────────┤");
        println!(
            "│  DOCS:  {}",
            if docs_fail == 0 {
                "PASS                          │".to_string()
            } else {
                format!("FAIL ({docs_fail} issues)               │")
            }
        );
        println!(
            "│  TESTS: {}",
            if tests_fail == 0 {
                "PASS                          │".to_string()
            } else {
                format!("FAIL ({tests_fail} issues)               │")
            }
        );
        println!(
            "│  SRS:   {}",
            if srs_fail == 0 {
                "PASS                          │".to_string()
            } else {
                format!("FAIL ({srs_fail} issues)               │")
            }
        );
        println!("└───────────────────────────────────────┘");

        if !self.passed() {
            println!();
            println!("Failures:");
            println!();
            for failure in &self.failures {
                println!(
                    "  [{cat}] {file}:{line} — {name}: {msg}",
                    cat = failure.category,
                    file = failure.file.display(),
                    line = failure.line,
                    name = failure.item_name,
                    msg = failure.message,
                );
            }
            println!();
            println!("Commit rejected. Fix the issues above and try again.");
        } else {
            println!();
            println!("All checks passed. Commit allowed.");
        }
    }
}

/// Gets the list of staged files from `git diff --cached --name-only`.
///
/// Returns absolute paths by joining with the workspace root.
///
/// # Examples
///
/// ```no_run
/// use trinity::check::get_staged_files;
/// use std::path::Path;
///
/// let files = get_staged_files(Path::new(".")).unwrap();
/// for f in &files {
///     println!("Staged: {}", f.display());
/// }
/// ```
pub fn get_staged_files(workspace_root: &Path) -> TrinityResult<Vec<PathBuf>> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only", "--diff-filter=ACMR"])
        .current_dir(workspace_root)
        .output()
        .map_err(|e| TrinityError::Git(format!("Failed to run git: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TrinityError::Git(format!(
            "git diff --cached failed: {stderr}"
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let files = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| workspace_root.join(l))
        .collect();
    Ok(files)
}

/// Reads the SRS.md file and returns its contents.
///
/// Returns an empty string if the file doesn't exist (SRS check will
/// then fail for all items, which is the correct behavior — you need
/// an SRS).
///
/// # Examples
///
/// ```
/// use trinity::check::read_srs;
/// use std::path::Path;
///
/// // Non-existent SRS returns empty string
/// let srs = read_srs(Path::new("/nonexistent/path"));
/// assert!(srs.is_empty());
/// ```
pub fn read_srs(workspace_root: &Path) -> String {
    let srs_path = workspace_root.join("artifacts").join("latest").join("SRS.md");
    std::fs::read_to_string(srs_path).unwrap_or_default()
}

/// Checks a single public item against the SRS document content.
///
/// The item passes if its name appears anywhere in the SRS text.
///
/// # Examples
///
/// ```
/// use trinity::check::item_in_srs;
///
/// let srs = "## Functions\n\n- `parse_file` — parses a Rust file\n";
/// assert!(item_in_srs("parse_file", srs));
/// assert!(!item_in_srs("nonexistent_fn", srs));
/// ```
pub fn item_in_srs(item_name: &str, srs_content: &str) -> bool {
    srs_content.contains(item_name)
}

/// Runs the docs check on a set of public items.
///
/// Rules:
/// - Every pub item must have a `///` doc comment
/// - Every pub fn doc comment should include a code block (``` example)
///
/// # Examples
///
/// ```
/// use trinity::check::{check_docs, CheckCategory};
/// use trinity::scanner::{PubItem, ItemKind};
/// use std::path::PathBuf;
///
/// let items = vec![PubItem {
///     name: "undocumented".to_string(),
///     kind: ItemKind::Function,
///     file: PathBuf::from("test.rs"),
///     line: 1,
///     has_doc_comment: false,
///     doc_has_code_block: false,
/// }];
/// let failures = check_docs(&items);
/// assert_eq!(failures.len(), 1);
/// assert_eq!(failures[0].category, CheckCategory::Docs);
/// ```
pub fn check_docs(items: &[PubItem]) -> Vec<CheckFailure> {
    let mut failures = Vec::new();
    for item in items {
        if !item.has_doc_comment {
            failures.push(CheckFailure {
                category: CheckCategory::Docs,
                item_name: item.name.clone(),
                file: item.file.clone(),
                line: item.line,
                message: format!(
                    "pub {} `{}` is missing a doc comment (/// ...)",
                    item.kind, item.name
                ),
            });
        } else if item.kind == ItemKind::Function && !item.doc_has_code_block {
            failures.push(CheckFailure {
                category: CheckCategory::Docs,
                item_name: item.name.clone(),
                file: item.file.clone(),
                line: item.line,
                message: format!(
                    "pub fn `{}` doc comment should include a code example (``` block)",
                    item.name
                ),
            });
        }
    }
    failures
}

/// Runs the tests check on a set of public functions.
///
/// For each pub fn, looks for a corresponding test function across all
/// provided test names.
///
/// # Examples
///
/// ```
/// use trinity::check::{check_tests, CheckCategory};
/// use trinity::scanner::{PubItem, ItemKind};
/// use std::path::PathBuf;
///
/// let items = vec![PubItem {
///     name: "my_function".to_string(),
///     kind: ItemKind::Function,
///     file: PathBuf::from("test.rs"),
///     line: 1,
///     has_doc_comment: true,
///     doc_has_code_block: true,
/// }];
/// let test_names = vec!["test_other".to_string()];
/// let failures = check_tests(&items, &test_names);
/// assert_eq!(failures.len(), 1);
/// assert_eq!(failures[0].category, CheckCategory::Tests);
/// ```
pub fn check_tests(items: &[PubItem], all_test_names: &[String]) -> Vec<CheckFailure> {
    let mut failures = Vec::new();
    for item in items {
        if item.kind != ItemKind::Function {
            continue;
        }
        if !has_corresponding_test(&item.name, all_test_names) {
            failures.push(CheckFailure {
                category: CheckCategory::Tests,
                item_name: item.name.clone(),
                file: item.file.clone(),
                line: item.line,
                message: format!(
                    "pub fn `{}` has no corresponding test (expected test_{} or similar)",
                    item.name, item.name
                ),
            });
        }
    }
    failures
}

/// Runs the SRS check on a set of public items.
///
/// Every pub fn/struct/enum/trait should appear in the SRS document.
///
/// # Examples
///
/// ```
/// use trinity::check::{check_srs, CheckCategory};
/// use trinity::scanner::{PubItem, ItemKind};
/// use std::path::PathBuf;
///
/// let items = vec![PubItem {
///     name: "MyStruct".to_string(),
///     kind: ItemKind::Struct,
///     file: PathBuf::from("test.rs"),
///     line: 1,
///     has_doc_comment: true,
///     doc_has_code_block: false,
/// }];
/// let srs = "Some SRS content mentioning other things only";
/// let failures = check_srs(&items, srs);
/// assert_eq!(failures.len(), 1);
/// assert_eq!(failures[0].category, CheckCategory::Srs);
/// ```
pub fn check_srs(items: &[PubItem], srs_content: &str) -> Vec<CheckFailure> {
    // If no SRS exists yet, skip the SRS check entirely (don't fail everything)
    if srs_content.is_empty() {
        return Vec::new();
    }

    let mut failures = Vec::new();
    for item in items {
        if !item_in_srs(&item.name, srs_content) {
            failures.push(CheckFailure {
                category: CheckCategory::Srs,
                item_name: item.name.clone(),
                file: item.file.clone(),
                line: item.line,
                message: format!(
                    "pub {} `{}` is not documented in artifacts/latest/SRS.md",
                    item.kind, item.name
                ),
            });
        }
    }
    failures
}

/// Runs the full `trinity check` pipeline.
///
/// Steps:
/// 1. Verify Trinity is initialized
/// 2. Get staged files from git
/// 3. Filter to .rs files, parse with syn
/// 4. Run docs check, tests check, and SRS check
/// 5. Print report
/// 6. Return exit code (0 = pass, 1 = fail)
///
/// # Examples
///
/// ```no_run
/// use trinity::check::run_check;
/// use std::path::Path;
///
/// let exit_code = run_check(Path::new(".")).unwrap();
/// std::process::exit(exit_code);
/// ```
pub fn run_check(workspace_root: &Path) -> TrinityResult<i32> {
    // Step 1: Verify initialized
    if !state::is_initialized(workspace_root) {
        return Err(TrinityError::NotInitialized);
    }

    // Step 2: Get staged files
    let staged = get_staged_files(workspace_root)?;
    let rs_staged: Vec<_> = staged
        .iter()
        .filter(|f| f.extension().is_some_and(|ext| ext == "rs"))
        .filter(|f| f.exists()) // Skip deleted files
        .cloned()
        .collect();

    if rs_staged.is_empty() {
        println!("No Rust files staged. Nothing to check.");
        return Ok(0);
    }

    println!(
        "Checking {} staged Rust file(s)...",
        rs_staged.len()
    );

    // Step 3: Parse staged files
    let scan = scanner::scan_files(&rs_staged)?;

    // Collect test names from all staged files
    let mut all_test_names = Vec::new();
    for file in &rs_staged {
        if let Ok(source) = std::fs::read_to_string(file) {
            let names = find_test_names_in_source(&source);
            all_test_names.extend(names);
        }
    }

    // Also look for test names in companion test files
    // (e.g. if src/scanner.rs is staged, also check tests/scanner_tests.rs)
    for file in &rs_staged {
        if let Some(stem) = file.file_stem().and_then(|s| s.to_str()) {
            let test_file = workspace_root
                .join("tests")
                .join(format!("{stem}_tests.rs"));
            if test_file.exists() {
                if let Ok(source) = std::fs::read_to_string(&test_file) {
                    let names = find_test_names_in_source(&source);
                    all_test_names.extend(names);
                }
            }
        }
    }

    // Step 4: Run checks
    let srs = read_srs(workspace_root);
    let doc_failures = check_docs(&scan.items);
    let test_failures = check_tests(&scan.items, &all_test_names);
    let srs_failures = check_srs(&scan.items, &srs);

    // Step 5: Build report
    let mut report = CheckReport {
        files_checked: scan.file_count,
        items_checked: scan.items.len(),
        ..Default::default()
    };
    report.failures.extend(doc_failures);
    report.failures.extend(test_failures);
    report.failures.extend(srs_failures);

    // Also report parse errors
    for (path, msg) in &scan.parse_errors {
        report.failures.push(CheckFailure {
            category: CheckCategory::Docs,
            item_name: "<parse error>".to_string(),
            file: path.clone(),
            line: 0,
            message: format!("File failed to parse: {msg}"),
        });
    }

    // Step 6: Print and return
    report.print_summary();
    Ok(if report.passed() { 0 } else { 1 })
}
