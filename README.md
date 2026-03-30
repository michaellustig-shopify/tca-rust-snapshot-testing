# rust-snapshot-testing

Rust port of [Point-Free's swift-snapshot-testing](https://github.com/pointfreeco/swift-snapshot-testing) -- inline and file-based snapshot testing.

## What This Is

A snapshot testing library for Rust:

- **`snapshot-testing`** -- Core snapshot assertion library
- **`inline-snapshot-testing`** -- Inline snapshot assertions (updates test source on failure)
- **`snapshot-testing-custom-dump`** -- Pretty-print and diff integration

## Quick Start

```bash
# Run all tests
cargo test

# Lint
cargo clippy

# Generate docs
cargo doc --open
```

## Crate Structure

| Crate | Purpose |
|-------|---------|
| `snapshot-testing` | Core snapshot assertion library |
| `inline-snapshot-testing` | Inline snapshots that update source code |
| `snapshot-testing-custom-dump` | Custom dump integration for readable diffs |

## Usage with TCA

Used by the [TCA Rust Port](https://github.com/michaellustig-shopify/tca-rust-port) and other sibling crates for test assertions. The `TestStore` in TCA uses snapshot testing for exhaustive state verification.

## Credits

- [Point-Free](https://www.pointfree.co/) -- original swift-snapshot-testing

## License

MIT
