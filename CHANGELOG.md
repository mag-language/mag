# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.9.1] - September 13, 2026

### Changed

- Value patterns in method signatures must be literals, including negative numbers; expression patterns like `def f(1 + 2)` are a compile error.
- Parentheses containing only record fields, like `p: (x: 1)`, are nested records.

### Fixed

- `match` arms now correctly compare strings, booleans, `nothing`, value expressions like `1 + 2`, typed record fields, nested records and tuples. Previously such arms matched any value, and tuple subjects failed to compile.
- Multimethod variants with equal precedence are tried in definition order instead of an arbitrary order.

## [0.9.0] - September 13, 2026

### Added

- Everything new in `magc` 0.9.0 is available in the REPL and when running files: `match` expressions, records, `nothing`, multi-line definitions, unary minus, and `~` concatenation.
- One-liner `match` expressions no longer need `end`, e.g. `match 3 case 7 then 1`.
- The REPL echoes the result of every expression, including `if`, `match`, literals, and `nothing`. `def`, `var`, and `print` stay silent.
- Example scripts in `scripts/` demonstrating multimethod dispatch, named record fields, and `match`.

### Changed

- The REPL and file runner no longer manage a runtime dispatch table; dispatch is resolved at compile time.
- The REPL greets you with an ASCII magpie instead of the text logo.

### Fixed

- Multimethod variants that differ only in a literal value (e.g. `fib(0)` and `fib(1)`) can be defined on separate REPL lines.
- Calling a multimethod with an unsupported argument now shows an error instead of silently stopping.
- `print(nothing)` prints `nothing` instead of producing no output.
- The REPL no longer fails to start when launched outside the project directory, e.g. after `cargo install mag_lang`; the startup banner is now embedded in the binary.

## [0.8.0] - May 10, 2026

### Added

- File execution: `mag run <file>` compiles and runs a `.mag` source file.
- Proper multi-line REPL rendering: input that wraps past the terminal width no longer causes display drift. The renderer tracks the cursor row, moves up by the correct number of wrapped lines, and clears downward before each redraw.

### Fixed

- Auto-printing of top-level expression results is now gated behind `repl_mode` and no longer triggers when running source files.

## [0.7.0] - May 9, 2026

### Added

- REPL syntax highlighting for keywords, type names, strings, numbers, operators, and punctuation, with a `MAG_REPL_THEME` environment variable (`mag` or `mono`/`plain`) to switch between the default colored theme and a monochrome theme.
- A `--debug` CLI flag that prints compiled instructions and VM register state on each REPL line.

### Fixed

- Multimethod definitions made in one REPL line are now correctly re-registered before each subsequent execution, fixing errors when calling recursive multimethods across lines.
