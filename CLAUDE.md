# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`woxml` is a write-only XML library for Rust focused on efficient XML generation without DOM structures or intermediate representations. It supports both `std` and `no_std` environments (via `alloc`).

## Common Commands

```bash
# Build
cargo build
cargo build --release

# Test
cargo test                        # std tests
cargo test --all-features
cargo test --no-default-features  # test no_std mode

# Lint (strict lints enforced in Cargo.toml)
cargo clippy
cargo fmt --check

# Benchmarks
cargo bench

# Coverage (requires nightly + cargo-llvm-cov)
cargo make cov          # HTML report
cargo make cov-summary  # JSON summary

# Single test
cargo test test_name
```

## Architecture

The library is small (~500 lines of core logic) with four source files:

- [src/lib.rs](src/lib.rs) — re-exports public types
- [src/woxml.rs](src/woxml.rs) — `XmlWriter<'a, W: Write>`, the main type
- [src/write.rs](src/write.rs) — custom `Write` trait (no_std-compatible substitute for `std::io::Write`)
- [src/error.rs](src/error.rs) — `Error` enum using `thiserror` with `no_std` support

**XmlWriter** is parameterized over a buffer type implementing the custom `Write` trait. Implementations exist for `Vec<u8>` and `bytes::BytesMut`. The writer maintains a stack of open elements and a namespace stack to track nesting and produce correct XML.

Two output modes: `compact_mode()` (minified) and `pretty_mode()` (indented). Methods chain builder-style: `begin_elem()` → `attr()` / `text()` → `end_elem()`.

## Features / no_std

The `std` feature (default) enables standard library support for `bytes` and `thiserror`. Disable it for embedded targets:

```toml
woxml = { version = "...", default-features = false }
```

## Lint Rules

Cargo.toml enforces `unwrap_used = "deny"`, `expect_used = "deny"`, `panic = "deny"`, and Clippy `pedantic` + `nursery`. All fallible operations must return `Result`.

## Embedded Support

The [embedded/ariel-os/](embedded/ariel-os/) directory contains benchmarks, tests, and usage examples for ARM Cortex-M targets via the ariel-os framework. Builds use the `laze` build system rather than plain Cargo.
