# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-04-13

### Changed

- Pinned the GitHub release workflow to Rust 1.92.0 and `--locked` builds for release artifacts

## [0.2.0] - 2026-02-19

### Added

- Separate `instances_dir` config option for Prism instances stored outside the main PrismLauncher data directory

### Changed

- `instances_dir` validation now runs alongside `data_dir` validation for consistent startup errors

## [0.1.0] - 2025-01-24

### Added

- Initial release of prism-tui
- Instance browsing with groups, versions, and mod loader information
- Instance launching with optional server auto-join
- Server management (add, edit, delete servers per instance)
- Join-on-launch configuration for servers
- Account selection and switching
- Log viewer for instance and launcher logs
- Incremental search for instances and accounts
- Vim-style keybindings with toggle for arrow key mode
- Help screen with keybinding reference
