# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html),
especially the [Rust flavour](https://doc.rust-lang.org/cargo/reference/semver.html).

## [Schema] - 2025-??-??

### Added

### Changed

### Fixed

### Removed

## [0.4.0] - 2025-??-??

### Added

### Changed
- Error messages are non_exhaustive
- no longer panics but errors when closing more elements/namespaces than opened
- no longer panics but errors when writing without an open element

### Fixed

### Removed

## [0.3.0] - 2025-08-05

### Added
- 'bytes::BytesMut' as writer target

### Changed
- externalized tests to better catch breaking changes

### Fixed
- make Error type & Write trait public

### Removed
- usage of 'std::io::Write' types as writer targets

## [0.2.0] - 2025-08-04

### Fixed
- changing the error type in 'Result<...>' is a breaking change

## [0.1.2] - 2025-08-04

### Added
- more documentation
- embedded environment with ariel-os
- feature "std" by default

### Changed
- crate is now 'no_std'
- performance improvement
- now has its own trait 'Write'

### Fixed
- clippy complaints

## [0.1.1] - 2025-07-30

### Added
- missing namespace methods

### Changed
- cleanup old pretty mode
- reduce code duplicates

### Fixed
- pretty output of text content
- visibility of internal namespace field

## [0.1.0] - 2025-07-23

Version 0.1.0 is a "takeover" of the xml_writer crate with some changes:

### Changed
- refactor coding to current `Rust` standards
- provide dedicated constructors and mode setters
- pretty mode puts also the end-tags in separate lines


### Removed
- remove public access to internal variables
