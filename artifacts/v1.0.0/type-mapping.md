# Type Mapping

## rust-snapshot-testing v1.0.0

Complete mapping of every Swift type, protocol, closure, and generic to its Rust equivalent.

---

## Table of Contents

1. [Core Types](#1-core-types)
2. [Configuration Types](#2-configuration-types)
3. [Protocol to Trait Mappings](#3-protocol-to-trait-mappings)
4. [Closure Type Mappings](#4-closure-type-mappings)
5. [Generics and Associated Types](#5-generics-and-associated-types)
6. [Standard Library Type Mappings](#6-standard-library-type-mappings)
7. [Error Types](#7-error-types)
8. [Platform-Specific Type Aliases](#8-platform-specific-type-aliases)
9. [Inline Snapshot Types](#9-inline-snapshot-types)
10. [Type Aliases](#10-type-aliases)

---

## 1. Core Types

### 1.1 Snapshotting

| Swift | Rust | Location |
|-------|------|----------|
| `Snapshotting<Value, Format>` | `Snapshotting<V, F>` | `snapshotting.rs` |

#### Fields

| Swift Field | Swift Type | Rust Field | Rust Type |
|-------------|-----------|------------|-----------|
| `pathExtension` | `String?` | `path_extension` | `Option<String>` |
| `diffing` | `Diffing<Format>` | `diffing` | `Diffing<F>` |
| `snapshot` | `(Value) -> Async<Format>` | `snapshot` | `Arc<dyn Fn(&V) -> Pin<Box<dyn Future<Output = F> + Send>> + Send + Sync>` |

#### Constraints

| Swift | Rust |
|-------|------|
| (none on Value) | `V: 'static` |
| (none on Format) | `F: Clone + Send + 'static` |

### 1.2 Diffing

| Swift | Rust | Location |
|-------|------|----------|
| `Diffing<Value>` | `Diffing<V>` | `diffing.rs` |

#### Fields

| Swift Field | Swift Type | Rust Field | Rust Type |
|-------------|-----------|------------|-----------|
| `toData` | `(Value) -> Data` | `to_data` | `Arc<dyn Fn(&V) -> Vec<u8> + Send + Sync>` |
| `fromData` | `(Data) -> Value` | `from_data` | `Arc<dyn Fn(&[u8]) -> V + Send + Sync>` |
| `diffV2` | `(Value, Value) -> (String, [DiffAttachment])?` | `diff` | `Arc<dyn Fn(&V, &V) -> Option<(String, Vec<DiffAttachment>)> + Send + Sync>` |
| `diff` (deprecated) | `(Value, Value) -> (String, [XCTAttachment])?` | NOT PORTED | Deprecated in Swift |

#### Constraints

| Swift | Rust |
|-------|------|
| (none) | `V: Clone` |

### 1.3 Async

| Swift | Rust | Notes |
|-------|------|-------|
| `Async<Value>` | `Pin<Box<dyn Future<Output = V> + Send>>` | Full replacement |
| `Async.init(run:)` | `async { ... }` block or `Box::pin(async { ... })` | Callback to future |
| `Async.init(value:)` | `std::future::ready(v)` then `Box::pin(...)` | Immediate value |
| `Async.map(_:)` | `future.map(transform)` via `FutureExt` | Requires `futures` crate |
| `async.run { callback in ... }` | `future.await` | Execute the async operation |

### 1.4 DiffAttachment

| Swift | Rust |
|-------|------|
| `enum DiffAttachment` | `enum DiffAttachment` |
| `.data(Data, name: String)` | `Data { bytes: Vec<u8>, name: String }` |
| `.xcTest(XCTAttachment)` | NOT PORTED (deprecated in Swift) |

---

## 2. Configuration Types

### 2.1 SnapshotTestingConfiguration

| Swift | Rust |
|-------|------|
| `struct SnapshotTestingConfiguration` | `struct SnapshotTestingConfiguration` |

| Swift Field | Swift Type | Rust Field | Rust Type |
|-------------|-----------|------------|-----------|
| `record` | `Record?` | `record` | `Option<Record>` |
| `diffTool` | `DiffTool?` | `diff_tool` | `Option<DiffTool>` |

| Swift Static | Rust |
|-------------|------|
| `@TaskLocal static var current: Self?` | `thread_local! { static CONFIG_STACK: RefCell<Vec<...>> }` |

### 2.2 Record

| Swift | Rust |
|-------|------|
| `struct Record` (with private enum Storage) | `enum Record` |
| `.all` | `Record::All` |
| `.failed` | `Record::Failed` |
| `.missing` | `Record::Missing` |
| `.never` | `Record::Never` |

| Swift Method | Rust Method |
|-------------|-------------|
| `init?(rawValue: String)` | `Record::from_str(&str) -> Option<Self>` |
| `ExpressibleByBooleanLiteral` | NOT PORTED (deprecated in Swift) |

### 2.3 DiffTool

| Swift | Rust |
|-------|------|
| `struct DiffTool` | `struct DiffTool` |

| Swift Field | Swift Type | Rust Field | Rust Type |
|-------------|-----------|------------|-----------|
| `tool` | `@Sendable (String, String) -> String` | `command` | `Arc<dyn Fn(&str, &str) -> String + Send + Sync>` |

| Swift Method/Init | Rust |
|------------------|------|
| `init(_ tool:)` | `DiffTool::new(closure)` |
| `init(stringLiteral:)` | `DiffTool::from_command(cmd_name)` (prepends command to args) |
| `callAsFunction(currentFilePath:failedFilePath:)` | `diff_tool.command(current, failed)` |
| `.ksdiff` | `DiffTool::ksdiff()` |
| `.default` | `DiffTool::default_tool()` |
| `ExpressibleByNilLiteral` | NOT PORTED (deprecated in Swift) |

---

## 3. Protocol to Trait Mappings

### 3.1 Direct Protocol to Trait

| Swift Protocol | Rust Trait | Notes |
|---------------|-----------|-------|
| `AnySnapshotStringConvertible` | `SnapshotDisplay` | Optional trait for custom dump |
| `Sendable` | `Send + Sync` | Thread safety marker |
| `Equatable` | `PartialEq + Eq` | Equality comparison |
| `Hashable` | `Hash + Eq` | Hashing |
| `CustomStringConvertible` | `std::fmt::Display` | Text representation |
| `CustomDebugStringConvertible` | `std::fmt::Debug` | Debug representation |
| `Encodable` | `serde::Serialize` | Serialization |
| `Decodable` | `serde::Deserialize` | Deserialization |
| `CaseIterable` | `strum::IntoEnumIterator` or custom iterator | All cases of an enum |
| `ExpressibleByStringLiteral` | `From<&str>` | String literal conversion |

### 3.2 Swift Testing Protocols (Not Ported)

| Swift Protocol | Why Not Ported |
|---------------|---------------|
| `SuiteTrait` | Swift Testing framework specific |
| `TestTrait` | Swift Testing framework specific |
| `TestScoping` | Swift Testing framework specific |
| `XCTestObservation` | XCTest specific |

### 3.3 SnapshotDisplay Trait

```swift
// Swift
protocol AnySnapshotStringConvertible {
    static var renderChildren: Bool { get }
    var snapshotDescription: String { get }
}
```

```rust
// Rust
pub trait SnapshotDisplay {
    fn snapshot_description(&self) -> String;
    fn render_children() -> bool { false }
}
```

| Swift | Rust |
|-------|------|
| `static var renderChildren: Bool` | `fn render_children() -> bool` (default `false`) |
| `var snapshotDescription: String` | `fn snapshot_description(&self) -> String` |

---

## 4. Closure Type Mappings

### 4.1 Snapshot Function

```swift
// Swift
var snapshot: (Value) -> Async<Format>
```

```rust
// Rust
Arc<dyn Fn(&V) -> Pin<Box<dyn Future<Output = F> + Send>> + Send + Sync>
```

Decomposition:

| Component | Swift | Rust | Why |
|-----------|-------|------|-----|
| Wrapper | (implicit ARC) | `Arc<...>` | Cloneable, shared ownership |
| Trait bound | (implicit closure) | `dyn Fn(...)` | Dynamic dispatch for type erasure |
| Input | `Value` (owned) | `&V` (borrowed) | Avoid requiring Clone on V |
| Output wrapper | `Async<Format>` | `Pin<Box<dyn Future<Output = F> + Send>>` | Async execution |
| Thread safety | `@Sendable` | `+ Send + Sync` | Cross-thread usage |

### 4.2 Diff Function

```swift
// Swift
var diffV2: (Value, Value) -> (String, [DiffAttachment])?
```

```rust
// Rust
Arc<dyn Fn(&V, &V) -> Option<(String, Vec<DiffAttachment>)> + Send + Sync>
```

### 4.3 Serialization Functions

```swift
// Swift
var toData: (Value) -> Data
var fromData: (Data) -> Value
```

```rust
// Rust
Arc<dyn Fn(&V) -> Vec<u8> + Send + Sync>
Arc<dyn Fn(&[u8]) -> V + Send + Sync>
```

### 4.4 DiffTool Command

```swift
// Swift
var tool: @Sendable (_ currentFilePath: String, _ failedFilePath: String) -> String
```

```rust
// Rust
Arc<dyn Fn(&str, &str) -> String + Send + Sync>
```

### 4.5 Pullback Transform

```swift
// Swift (sync)
@escaping (_ otherValue: NewValue) -> Value

// Swift (async)
@escaping (_ otherValue: NewValue) -> Async<Value>
```

```rust
// Rust (sync)
impl Fn(&NewV) -> V + Send + Sync + 'static

// Rust (async)
impl Fn(&NewV) -> Pin<Box<dyn Future<Output = V> + Send>> + Send + Sync + 'static
```

### 4.6 Configuration Scope Closure

```swift
// Swift
func withSnapshotTesting<R>(operation: () throws -> R) rethrows -> R
```

```rust
// Rust
pub fn with_snapshot_testing<R>(config: ..., f: impl FnOnce() -> R) -> R
```

| Aspect | Swift | Rust |
|--------|-------|------|
| Closure trait | implicit | `FnOnce() -> R` |
| Error handling | `throws/rethrows` | (panics propagate naturally) |
| Return type | generic `R` | generic `R` |

---

## 5. Generics and Associated Types

### 5.1 Type Parameter Naming

| Swift Convention | Rust Convention | Used For |
|-----------------|----------------|----------|
| `Value` | `V` | The type being snapshotted |
| `Format` | `F` | The diffable format type |
| `NewValue` | `NewV` | Pullback target type |
| `R` | `R` | Generic return type |

### 5.2 Generic Constraints

| Swift | Rust | Context |
|-------|------|---------|
| (none) | `V: 'static` | Stored in Arc closures |
| (none) | `F: Clone + Send + 'static` | Cloned during comparison, shared across threads |
| `where Value == Format` | `impl<F: Clone> Snapshotting<F, F>` | SimplySnapshotting |
| `where Value: Encodable` | `impl<V: serde::Serialize>` | JSON strategy |
| `where Value: CaseIterable` | `impl<V: IntoIterator>` | Function strategy |
| `where Format == String` | `impl<V> Snapshotting<V, String>` | Text-based strategies |
| `where Value == String, Format == String` | `impl Snapshotting<String, String>` | Lines strategy |
| `where Value == Data, Format == Data` | `impl Snapshotting<Vec<u8>, Vec<u8>>` | Data strategy |

### 5.3 Generic Extensions (Static Methods on Constrained Types)

Swift uses constrained extensions to add static methods:

```swift
extension Snapshotting where Value: Encodable, Format == String {
    static var json: Snapshotting { ... }
}
```

Rust uses constrained impl blocks:

```rust
impl<V: serde::Serialize> Snapshotting<V, String> {
    pub fn json() -> Self { ... }
}
```

Full mapping of constrained extensions:

| Swift Extension Constraint | Rust impl Block | Strategy |
|---------------------------|----------------|----------|
| `Value == String, Format == String` | `impl Snapshotting<String, String>` | `.lines` |
| `Value == Data, Format == Data` | `impl Snapshotting<Vec<u8>, Vec<u8>>` | `.data` |
| `Value: Encodable, Format == String` | `impl<V: Serialize> Snapshotting<V, String>` | `.json` |
| `Format == String` (any Value) | `impl<V> Snapshotting<V, String>` | `.description`, `.dump` |
| `Value: CaseIterable, Format == String` | `impl<V: IntoEnumIterator> Snapshotting<V, String>` | `.func()` |
| `Value == URLRequest, Format == String` | `impl Snapshotting<http::Request<B>, String>` | `.raw`, `.curl` |
| `Value == String` (Diffing) | `impl Diffing<String>` | `.lines` |

---

## 6. Standard Library Type Mappings

### 6.1 Foundation to std

| Swift (Foundation) | Rust (std/crates) | Notes |
|-------------------|-------------------|-------|
| `String` | `String` | Direct |
| `String?` | `Option<String>` | Direct |
| `Data` | `Vec<u8>` | Byte buffer |
| `URL` | `std::path::PathBuf` (file) / `url::Url` (http) | Split based on usage |
| `TimeInterval` | `std::time::Duration` | `5.0` becomes `Duration::from_secs(5)` |
| `[T]` (Array) | `Vec<T>` | Direct |
| `[K: V]` (Dictionary) | `HashMap<K, V>` or `BTreeMap<K, V>` | BTreeMap for sorted output |
| `Set<T>` | `HashSet<T>` or `BTreeSet<T>` | BTreeSet for sorted output |
| `NSLock` | `std::sync::Mutex<T>` | Or `parking_lot::Mutex` for non-poisoning |
| `NSRegularExpression` | `regex::Regex` | `regex` crate |
| `ProcessInfo.processInfo.environment` | `std::env::var(key)` | Environment variable access |
| `FileManager.default` | `std::fs` functions | File system operations |
| `DispatchQueue` | `tokio::task` or `std::thread` | Async dispatch |
| `JSONEncoder` | `serde_json::to_string_pretty` | JSON serialization |
| `PropertyListEncoder` | NOT PORTED | Apple-specific format |

### 6.2 XCTest to Rust Test

| Swift (XCTest) | Rust | Notes |
|---------------|------|-------|
| `XCTFail(message)` | `panic!(message)` | Test failure |
| `XCTestExpectation` | `tokio::sync::oneshot` or `Future` | Async wait |
| `XCTWaiter.wait(for:timeout:)` | `tokio::time::timeout(dur, fut)` | Timeout |
| `XCTestObservation` | Rust test harness hooks | Limited equivalent |
| `XCTContext.runActivity` | NOT PORTED | Xcode-specific |
| `XCTAttachment` | `DiffAttachment` | Renamed |
| `#file` | `file!()` | Source file path |
| `#fileID` | `module_path!()` + `file!()` | Module-qualified path |
| `#function` | `std::any::type_name::<T>()` or macro capture | Function name |
| `#line` | `line!()` | Source line number |
| `#column` | `column!()` | Source column number |
| `StaticString` | `&'static str` | Compile-time string |

### 6.3 Swift Concurrency to Rust

| Swift | Rust | Notes |
|-------|------|-------|
| `@TaskLocal` | `thread_local!` | Task-scoped vs thread-scoped storage |
| `@unchecked Sendable` | Manual `unsafe impl Send` | Override compiler checks |
| `Sendable` | `Send + Sync` | Thread safety |
| `async/await` | `async/.await` | Nearly identical syntax |

---

## 7. Error Types

### 7.1 Swift Error Model to Rust

| Swift Pattern | Rust Pattern | Notes |
|--------------|-------------|-------|
| `throws -> String?` (returns nil on success) | `Result<(), SnapshotError>` | `Ok(())` = success, `Err(...)` = failure |
| `LocalizedError` | `thiserror::Error` | Error with Display |
| `@autoclosure () throws -> Value` | `&V` (borrow) or `impl FnOnce() -> V` | Lazy evaluation |

### 7.2 SnapshotError Variants

| Error Condition | Swift Return | Rust Variant |
|----------------|-------------|-------------|
| Timeout | `String` (message) | `SnapshotError::Timeout { seconds: f64 }` |
| Couldn't snapshot | `String` (message) | `SnapshotError::SnapshotFailed` |
| Record mode, wrote file | `String` (message) | `SnapshotError::Recorded { path: PathBuf }` |
| Missing reference, never mode | `String` (message) | `SnapshotError::MissingSnapshot { path: PathBuf }` |
| Diff mismatch | `String` (message) | `SnapshotError::Mismatch { message: String, diff_tool_output: String }` |
| File I/O error | `error.localizedDescription` | `SnapshotError::Io(std::io::Error)` |

---

## 8. Platform-Specific Type Aliases

### 8.1 Swift Internal Type Aliases (NOT PORTED)

These are internal to the Swift library for cross-platform Apple compatibility:

```swift
// Swift (Internal.swift)
#if os(macOS)
  typealias Image = NSImage
  typealias View = NSView
#elseif os(iOS) || os(tvOS)
  typealias Image = UIImage
  typealias View = UIView
#endif
```

No Rust equivalent needed. All Apple UI types are out of scope.

### 8.2 Rust Platform Aliases

```rust
// For file path operations
#[cfg(target_os = "windows")]
const PATH_SEPARATOR: char = '\\';

#[cfg(not(target_os = "windows"))]
const PATH_SEPARATOR: char = '/';
```

---

## 9. Inline Snapshot Types

### 9.1 InlineSnapshotSyntaxDescriptor

| Swift | Rust |
|-------|------|
| `struct InlineSnapshotSyntaxDescriptor` | `struct InlineSnapshotSyntaxDescriptor` |

| Swift Field | Swift Type | Rust Field | Rust Type |
|-------------|-----------|------------|-----------|
| `trailingClosureLabel` | `String` | `trailing_closure_label` | `String` |
| `trailingClosureOffset` | `Int` | `trailing_closure_offset` | `usize` |
| `deprecatedTrailingClosureLabels` | `[String]` | `deprecated_trailing_closure_labels` | `Vec<String>` |

### 9.2 InlineSnapshot (Internal State)

| Swift | Rust |
|-------|------|
| `struct InlineSnapshot` | `struct InlineSnapshot` |

| Swift Field | Rust Field | Type |
|-------------|-----------|------|
| `expected` | `expected` | `Option<String>` |
| `actual` | `actual` | `Option<String>` |
| `wasRecording` | `was_recording` | `bool` |
| `syntaxDescriptor` | `syntax_descriptor` | `InlineSnapshotSyntaxDescriptor` |
| `function` | `function` | `String` |
| `line` | `line` | `u32` |
| `column` | `column` | `u32` |

### 9.3 File (Internal Key)

| Swift | Rust |
|-------|------|
| `struct File` (wraps `StaticString`) | `PathBuf` (used directly as HashMap key) |

### 9.4 LockIsolated

| Swift | Rust |
|-------|------|
| `class LockIsolated<Value>` | `std::sync::Mutex<Value>` |
| `lockIsolated.withLock { ... }` | `mutex.lock().unwrap()` or `parking_lot::Mutex` |

### 9.5 Source Rewriting Types (Swift Syntax to Rust syn)

| Swift Type | Rust Equivalent | Crate |
|-----------|----------------|-------|
| `SourceFileSyntax` | `syn::File` | `syn` |
| `FunctionCallExprSyntax` | `syn::ExprCall` / `syn::ExprMethodCall` | `syn` |
| `ClosureExprSyntax` | `syn::ExprClosure` | `syn` |
| `StringLiteralExprSyntax` | `syn::LitStr` | `syn` |
| `SourceLocationConverter` | manual line/column tracking | custom |
| `SyntaxRewriter` | `syn::visit_mut::VisitMut` | `syn` |
| `SyntaxVisitor` | `syn::visit::Visit` | `syn` |

---

## 10. Type Aliases

### 10.1 Direct Aliases

| Swift | Rust | Notes |
|-------|------|-------|
| `typealias SimplySnapshotting<Format> = Snapshotting<Format, Format>` | `type SimplySnapshotting<F> = Snapshotting<F, F>;` | When V == F |

### 10.2 Convenience Type Aliases (Rust-specific)

These don't exist in Swift but improve ergonomics in Rust:

```rust
/// A boxed, pinned, sendable future. Used throughout for async snapshot operations.
pub type SnapshotFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// The diff result type: None if equal, Some((message, attachments)) if different.
pub type DiffResult = Option<(String, Vec<DiffAttachment>)>;
```

### 10.3 Naming Convention Translation

| Swift Convention | Rust Convention | Example |
|-----------------|----------------|---------|
| camelCase (types) | PascalCase | `Snapshotting` -> `Snapshotting` |
| camelCase (fields/methods) | snake_case | `pathExtension` -> `path_extension` |
| camelCase (local vars) | snake_case | `snapshotFileUrl` -> `snapshot_file_url` |
| UPPER_CASE (statics) | SCREAMING_SNAKE_CASE | (same convention) |
| leading underscore (internal) | `pub(crate)` visibility | `_diffTool` -> `pub(crate) fn current_diff_tool()` |
| `@_spi(Internals)` | `#[doc(hidden)]` or `pub(crate)` | Semi-public API |
