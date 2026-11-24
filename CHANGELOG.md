# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.5](https://github.com/MikeLuu99/search-tui/compare/v0.1.4...v0.1.5) - 2026-11-24

### Fixed

- bound the search fan-out with a 3s deadline via `EngineLimits::with_deadline` (metadata-search-engine-rs 0.3.5): a hung upstream (e.g. DuckDuckGo slow-drip throttling) no longer stalls the UI for its full 8s engine timeout — measured 8.0s → 2.0s for the full search

### Added

- `examples/engine_timing.rs`: per-engine latency debug helper

## [0.1.4](https://github.com/MikeLuu99/search-tui/compare/v0.1.3...v0.1.4) - 2026-08-09

### Fixed

- add --version/--help CLI parsing with clap
