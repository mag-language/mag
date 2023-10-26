![mag banner](https://world-of-music.at/downloads/bird-banner.png)

# Introduction

Mag is an optionally typed, object-oriented programming language with patterns, classes and multimethods.

A simple example showing some code for a (very inefficent) recursive Fibonacci function can be defined like this using multimethods:

```python
def fib(0) 0
def fib(1) 1
def fib(n Int) fib(n - 2) + fib(n - 1)
```

This defines the same `fib` method multiple times, which is okay since the method signatures differ between the implementations, replacing the need for a separate conditional check inside the function.

More documentation will follow in the future, and make sure to check out [Robert Nystrom's blog posts about Magpie](https://journal.stuffwithstuff.com/category/magpie/) for further information. His posts serve as the foundational inspiration for this project.

# Features

### Extensibility

The parser and compiler structures use a modular, trait-based architecture with structs which handle the actual translation, which means that the actual interpretation of semantics is dynamic and may be extended by Mag code at runtime using [a special parselet](https://journal.stuffwithstuff.com/2011/02/13/extending-syntax-from-within-a-language/), a concept which could even be extended to compilation and execution to provide a very flexible programming environment.

### Pattern Matching

Patterns are not just used in Rust-like `match` expresions, they are actually dispersed throughout the whole language fabric of `mag` and `magc`, where it is used for method arguments, variable destructuring, error reporting and many other useful things.

# Getting Started

Enough fuzzy talk, let's start a REPL from the command line to get this project up and running.

For now, please make sure you have `magc` and `strontium` cloned into the same directory as our current runtime

# Roadmap

- [x] REPL
  - [x] Basic interface for entering commands
  - [ ] Cursor Movement
  - [ ] Syntax Highlighting with `syntect`
  - [ ] Error Reporting
    - [x] Simple error handling
    - [ ] Complex error messages with source code and help
  - [ ] Multi-Line Input
  - [ ] Subtask 1.2.2
- [ ] Main Task 2
  - [ ] Subtask 2.1
  - [ ] Subtask 2.2
- [x] Completed Main Task 3
  - [x] Completed Subtask 3.1
  - [ ] Subtask 3.2

