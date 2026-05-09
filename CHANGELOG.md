# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0] - May 9, 2026

### Added

- REPL syntax highlighting for keywords, type names, strings, numbers, operators, and punctuation, with a `MAG_REPL_THEME` environment variable (`mag` or `mono`/`plain`) to switch between the default colored theme and a monochrome theme.
- A `--debug` CLI flag that prints compiled instructions and VM register state on each REPL line.

### Fixed

- Multimethod definitions made in one REPL line are now correctly re-registered before each subsequent execution, fixing errors when calling recursive multimethods across lines.
