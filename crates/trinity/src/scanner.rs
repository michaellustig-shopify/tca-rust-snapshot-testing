//! ┌─────────────────────────────────────────────────────┐
//! │  scanner.rs — Rust source code scanner              │
//! ├─────────────────────────────────────────────────────┤
//! │  WHAT: Walks a directory tree for .rs files, parses │
//! │  each with `syn`, and extracts public items         │
//! │  (functions, structs, enums, traits) along with     │
//! │  whether they have doc comments and tests.          │
//! │                                                     │
//! │  WHY: Trinity needs to know what public API surface │
//! │  exists so it can verify docs and tests cover it.   │
//! │  `syn` gives us a proper AST instead of fragile     │
//! │  regex matching.                                    │
//! │                                                     │
//! │  ALTERNATIVES:                                      │
//! │  • tree-sitter — heavier dependency, overkill       │
//! │  • regex — misses nested items, false positives     │
//! │  • rust-analyzer — full LSP, way too heavy          │
//! │                                                     │
//! │  TESTED BY: doc tests below, scanner_tests.rs       │
//! │                                                     │
//! │  EDGE CASES: Non-UTF8 files, macro-generated code,  │
//! │  conditional compilation (#[cfg]), proc macros,     │
//! │  files that fail to parse.                          │
//! │                                                     │
//! │  CHANGELOG:                                         │
//! │  • v0.1.0 — Initial scanner with syn parsing        │
//! │                                                     │
//! │  HISTORY: git log --oneline --follow -- scanner.rs  │
//! └─────────────────────────────────────────────────────┘

use std::path::{Path, PathBuf};

use crate::error::{io_err, TrinityError, TrinityResult};

/// The kind of a public Rust item that Trinity tracks.
///
/// # Examples
///
/// ```
/// use trinity::scanner::ItemKind;
///
/// let kind = ItemKind::Function;
/// assert_eq!(format!("{:?}", kind), "Function");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemKind {
    /// A `pub fn`.
    Function,
    /// A `pub struct`.
    Struct,
    /// A `pub enum`.
    Enum,
    /// A `pub trait`.
    Trait,
}

impl std::fmt::Display for ItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemKind::Function => write!(f, "fn"),
            ItemKind::Struct => write!(f, "struct"),
            ItemKind::Enum => write!(f, "enum"),
            ItemKind::Trait => write!(f, "trait"),
        }
    }
}

/// A single public item extracted from a Rust source file.
///
/// # Examples
///
/// ```
/// use trinity::scanner::{PubItem, ItemKind};
/// use std::path::PathBuf;
///
/// let item = PubItem {
///     name: "my_function".to_string(),
///     kind: ItemKind::Function,
///     file: PathBuf::from("src/lib.rs"),
///     line: 42,
///     has_doc_comment: true,
///     doc_has_code_block: false,
/// };
/// assert_eq!(item.name, "my_function");
/// assert!(item.has_doc_comment);
/// ```
#[derive(Debug, Clone)]
pub struct PubItem {
    /// The identifier name (e.g. `my_function`, `MyStruct`).
    pub name: String,
    /// What kind of item this is.
    pub kind: ItemKind,
    /// The file this item was found in.
    pub file: PathBuf,
    /// The line number (1-based) where the item starts.
    pub line: usize,
    /// Whether the item has at least one `///` doc comment.
    pub has_doc_comment: bool,
    /// Whether any doc comment contains a code block (``` markers).
    pub doc_has_code_block: bool,
}

/// Results from scanning a workspace or a set of files.
///
/// # Examples
///
/// ```
/// use trinity::scanner::ScanResult;
///
/// let result = ScanResult::default();
/// assert_eq!(result.total_lines, 0);
/// assert!(result.items.is_empty());
/// ```
#[derive(Debug, Clone, Default)]
pub struct ScanResult {
    /// All public items found across all scanned files.
    pub items: Vec<PubItem>,
    /// Total number of .rs files scanned.
    pub file_count: usize,
    /// Total lines of Rust source code scanned.
    pub total_lines: usize,
    /// Files that failed to parse (path + error message).
    pub parse_errors: Vec<(PathBuf, String)>,
}

impl ScanResult {
    /// Returns the count of items that lack doc comments.
    ///
    /// # Examples
    ///
    /// ```
    /// use trinity::scanner::{ScanResult, PubItem, ItemKind};
    /// use std::path::PathBuf;
    ///
    /// let mut result = ScanResult::default();
    /// result.items.push(PubItem {
    ///     name: "documented".to_string(),
    ///     kind: ItemKind::Function,
    ///     file: PathBuf::from("a.rs"),
    ///     line: 1,
    ///     has_doc_comment: true,
    ///     doc_has_code_block: false,
    /// });
    /// result.items.push(PubItem {
    ///     name: "undocumented".to_string(),
    ///     kind: ItemKind::Function,
    ///     file: PathBuf::from("a.rs"),
    ///     line: 10,
    ///     has_doc_comment: false,
    ///     doc_has_code_block: false,
    /// });
    /// assert_eq!(result.undocumented_count(), 1);
    /// ```
    pub fn undocumented_count(&self) -> usize {
        self.items.iter().filter(|i| !i.has_doc_comment).count()
    }

    /// Estimates token usage for three AI passes over the scanned code.
    ///
    /// Formula: `total_lines * 4 tokens/line * 3 passes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use trinity::scanner::ScanResult;
    ///
    /// let mut result = ScanResult::default();
    /// result.total_lines = 100;
    /// assert_eq!(result.estimated_tokens(), 1200);
    /// ```
    pub fn estimated_tokens(&self) -> usize {
        self.total_lines * 4 * 3
    }
}

/// Collects all `.rs` files under a directory, skipping hidden dirs and target/.
///
/// # Examples
///
/// ```no_run
/// use trinity::scanner::find_rs_files;
/// use std::path::Path;
///
/// let files = find_rs_files(Path::new(".")).unwrap();
/// for f in &files {
///     println!("{}", f.display());
/// }
/// ```
pub fn find_rs_files(root: &Path) -> TrinityResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            // Skip hidden directories, target/, and .trinity/
            if e.file_type().is_dir() {
                return !name.starts_with('.') && name != "target";
            }
            true
        })
    {
        let entry = entry.map_err(|e| {
            let path = e.path().unwrap_or(Path::new("unknown")).to_path_buf();
            io_err(path, e.into())
        })?;
        if entry.file_type().is_file() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path.to_path_buf());
            }
        }
    }
    files.sort();
    Ok(files)
}

/// Checks whether a list of `syn` attributes contains any doc comment (`#[doc = "..."]`).
///
/// Returns `(has_doc, has_code_block)` — whether there is at least one doc
/// attribute, and whether any doc attribute contains a triple-backtick code block.
///
/// # Examples
///
/// ```
/// use trinity::scanner::check_doc_attrs;
///
/// // No attributes => no docs
/// let (has_doc, has_code) = check_doc_attrs(&[]);
/// assert!(!has_doc);
/// assert!(!has_code);
/// ```
pub fn check_doc_attrs(attrs: &[syn::Attribute]) -> (bool, bool) {
    let mut has_doc = false;
    let mut has_code_block = false;

    for attr in attrs {
        if attr.path().is_ident("doc") {
            has_doc = true;
            // Try to extract the doc string value
            if let syn::Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(expr_lit) = &meta.value {
                    if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                        let val = lit_str.value();
                        if val.contains("```") {
                            has_code_block = true;
                        }
                    }
                }
            }
        }
    }
    (has_doc, has_code_block)
}

/// Parses a single `.rs` file and extracts all public items.
///
/// Items extracted: `pub fn`, `pub struct`, `pub enum`, `pub trait`.
/// For each, records whether it has doc comments and code blocks.
///
/// Returns `Err(TrinityError::Parse)` if syn cannot parse the file.
/// Files that are not valid UTF-8 also produce a parse error.
///
/// # Examples
///
/// ```
/// use trinity::scanner::parse_file_items;
/// use std::path::Path;
///
/// // Parsing a non-existent file returns an IO error
/// let result = parse_file_items(Path::new("/nonexistent/file.rs"));
/// assert!(result.is_err());
/// ```
pub fn parse_file_items(path: &Path) -> TrinityResult<(Vec<PubItem>, usize)> {
    let source =
        std::fs::read_to_string(path).map_err(|e| io_err(path.to_path_buf(), e))?;
    let line_count = source.lines().count();

    let syntax = syn::parse_file(&source).map_err(|e| TrinityError::Parse {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;

    let mut items = Vec::new();
    extract_items_from_file(&syntax.items, path, &mut items);
    Ok((items, line_count))
}

/// Recursively extract public items from a list of syn items.
///
/// Handles top-level items and items nested inside `mod` blocks (inline modules).
/// Items inside `impl` blocks are also scanned for public methods.
fn extract_items_from_file(
    syn_items: &[syn::Item],
    path: &Path,
    out: &mut Vec<PubItem>,
) {
    for item in syn_items {
        match item {
            syn::Item::Fn(f) => {
                if matches!(f.vis, syn::Visibility::Public(_)) {
                    let (has_doc, has_code_block) = check_doc_attrs(&f.attrs);
                    out.push(PubItem {
                        name: f.sig.ident.to_string(),
                        kind: ItemKind::Function,
                        file: path.to_path_buf(),
                        line: line_of_span(f.sig.ident.span()),
                        has_doc_comment: has_doc,
                        doc_has_code_block: has_code_block,
                    });
                }
            }
            syn::Item::Struct(s) => {
                if matches!(s.vis, syn::Visibility::Public(_)) {
                    let (has_doc, has_code_block) = check_doc_attrs(&s.attrs);
                    out.push(PubItem {
                        name: s.ident.to_string(),
                        kind: ItemKind::Struct,
                        file: path.to_path_buf(),
                        line: line_of_span(s.ident.span()),
                        has_doc_comment: has_doc,
                        doc_has_code_block: has_code_block,
                    });
                }
            }
            syn::Item::Enum(e) => {
                if matches!(e.vis, syn::Visibility::Public(_)) {
                    let (has_doc, has_code_block) = check_doc_attrs(&e.attrs);
                    out.push(PubItem {
                        name: e.ident.to_string(),
                        kind: ItemKind::Enum,
                        file: path.to_path_buf(),
                        line: line_of_span(e.ident.span()),
                        has_doc_comment: has_doc,
                        doc_has_code_block: has_code_block,
                    });
                }
            }
            syn::Item::Trait(t) => {
                if matches!(t.vis, syn::Visibility::Public(_)) {
                    let (has_doc, has_code_block) = check_doc_attrs(&t.attrs);
                    out.push(PubItem {
                        name: t.ident.to_string(),
                        kind: ItemKind::Trait,
                        file: path.to_path_buf(),
                        line: line_of_span(t.ident.span()),
                        has_doc_comment: has_doc,
                        doc_has_code_block: has_code_block,
                    });
                }
            }
            syn::Item::Mod(m) => {
                // Recurse into inline modules
                if let Some((_, ref mod_items)) = m.content {
                    extract_items_from_file(mod_items, path, out);
                }
            }
            syn::Item::Impl(imp) => {
                // Extract pub methods from impl blocks
                for impl_item in &imp.items {
                    if let syn::ImplItem::Fn(method) = impl_item {
                        if matches!(method.vis, syn::Visibility::Public(_)) {
                            let (has_doc, has_code_block) =
                                check_doc_attrs(&method.attrs);
                            out.push(PubItem {
                                name: method.sig.ident.to_string(),
                                kind: ItemKind::Function,
                                file: path.to_path_buf(),
                                line: line_of_span(method.sig.ident.span()),
                                has_doc_comment: has_doc,
                                doc_has_code_block: has_code_block,
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Extracts a 1-based line number from a `proc_macro2::Span`.
///
/// Falls back to 0 if span information is not available (e.g. in tests
/// where span info is stripped).
fn line_of_span(span: proc_macro2::Span) -> usize {
    span.start().line
}

/// Scans an entire workspace: finds all `.rs` files, parses each, and
/// collects all public items into a single `ScanResult`.
///
/// Files that fail to parse are recorded in `parse_errors` but do not
/// abort the scan.
///
/// # Examples
///
/// ```no_run
/// use trinity::scanner::scan_workspace;
/// use std::path::Path;
///
/// let result = scan_workspace(Path::new(".")).unwrap();
/// println!("Found {} public items in {} files", result.items.len(), result.file_count);
/// ```
pub fn scan_workspace(root: &Path) -> TrinityResult<ScanResult> {
    let files = find_rs_files(root)?;
    let mut result = ScanResult {
        file_count: files.len(),
        ..Default::default()
    };

    for file in &files {
        match parse_file_items(file) {
            Ok((items, line_count)) => {
                result.total_lines += line_count;
                result.items.extend(items);
            }
            Err(TrinityError::Parse { path, message }) => {
                result.parse_errors.push((path, message));
            }
            Err(e) => {
                // IO errors for individual files are recorded but don't stop the scan
                result
                    .parse_errors
                    .push((file.clone(), format!("{e}")));
            }
        }
    }
    Ok(result)
}

/// Scans only the specified files (not a full workspace walk).
///
/// Useful for `trinity check` where we only care about staged files.
///
/// # Examples
///
/// ```no_run
/// use trinity::scanner::scan_files;
/// use std::path::PathBuf;
///
/// let files = vec![PathBuf::from("src/main.rs")];
/// let result = scan_files(&files).unwrap();
/// println!("Found {} public items", result.items.len());
/// ```
pub fn scan_files(files: &[PathBuf]) -> TrinityResult<ScanResult> {
    let rs_files: Vec<_> = files
        .iter()
        .filter(|f| f.extension().is_some_and(|ext| ext == "rs"))
        .collect();

    let mut result = ScanResult {
        file_count: rs_files.len(),
        ..Default::default()
    };

    for file in &rs_files {
        match parse_file_items(file) {
            Ok((items, line_count)) => {
                result.total_lines += line_count;
                result.items.extend(items);
            }
            Err(TrinityError::Parse { path, message }) => {
                result.parse_errors.push((path, message));
            }
            Err(e) => {
                result
                    .parse_errors
                    .push((file.to_path_buf(), format!("{e}")));
            }
        }
    }
    Ok(result)
}

/// Searches for test functions in a file's source code.
///
/// Returns a list of test function names found (functions annotated with
/// `#[test]` or `#[tokio::test]`).
///
/// # Examples
///
/// ```
/// use trinity::scanner::find_test_names_in_source;
///
/// let source = r#"
/// #[cfg(test)]
/// mod tests {
///     #[test]
///     fn test_something() {}
///
///     #[test]
///     fn it_works() {}
/// }
/// "#;
/// let names = find_test_names_in_source(source);
/// assert!(names.contains(&"test_something".to_string()));
/// assert!(names.contains(&"it_works".to_string()));
/// ```
pub fn find_test_names_in_source(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    if let Ok(syntax) = syn::parse_file(source) {
        collect_test_names(&syntax.items, &mut names);
    }
    names
}

/// Recursively collects names of functions annotated with `#[test]`.
fn collect_test_names(items: &[syn::Item], out: &mut Vec<String>) {
    for item in items {
        match item {
            syn::Item::Fn(f) => {
                if has_test_attr(&f.attrs) {
                    out.push(f.sig.ident.to_string());
                }
            }
            syn::Item::Mod(m) => {
                if let Some((_, ref mod_items)) = m.content {
                    collect_test_names(mod_items, out);
                }
            }
            _ => {}
        }
    }
}

/// Checks if any attribute in the list is `#[test]` or `#[tokio::test]`.
fn has_test_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("test")
            || attr
                .path()
                .segments
                .last()
                .is_some_and(|seg| seg.ident == "test")
    })
}

/// Checks whether a given public function name has a corresponding test.
///
/// The heuristic: look for a test function named `test_<fn_name>`, or
/// any test function whose name contains `<fn_name>`.
///
/// # Examples
///
/// ```
/// use trinity::scanner::has_corresponding_test;
///
/// let test_names = vec![
///     "test_parse_file".to_string(),
///     "it_scans_workspace".to_string(),
/// ];
/// assert!(has_corresponding_test("parse_file", &test_names));
/// assert!(!has_corresponding_test("missing_fn", &test_names));
/// ```
pub fn has_corresponding_test(fn_name: &str, test_names: &[String]) -> bool {
    let target = format!("test_{fn_name}");
    test_names.iter().any(|t| {
        t == &target || t.contains(fn_name)
    })
}
