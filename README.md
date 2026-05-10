![mag banner](https://world-of-music.at/downloads/bird-banner.png)

# Mag

Mag is an optionally typed, expression-oriented programming language with multimethods and pattern matching.

## Quick start

```
cargo run          # start the REPL
cargo run -- run <file.mag>   # run a source file
cargo run -- --debug          # REPL with instruction/register output
```

## Language overview

### Arithmetic and comparisons

```
>>> 1 + 2
3
>>> 10 / 4
2.5
>>> 3 > 2
true
```

### Multimethods

Define the same method name multiple times with different signatures. The runtime picks the right one based on the argument:

```
>>> def fib(0) 0
>>> def fib(1) 1
>>> def fib(n Int) fib(n - 1) + fib(n - 2)
>>> fib(10)
55
```

### Type-based dispatch

Type annotations on parameters create distinct dispatch entries:

```
>>> def double(n Int) n * 2
>>> def double(n String) n + n
>>> double(5)
10
>>> double("hi")
hihi
```

### Conditionals

```
>>> if 3 > 2 then print("yes") else print("no") end
yes
```

### Variables

```
>>> var x = 42
>>> x
42
```

Inside a method body, `var` binds a local variable:

```
>>> def greet(name String)
...   var msg = "hello " + name
...   msg
... end
>>> greet("world")
hello world
```

### Early return

```
>>> def abs(n Int)
...   if n < 0 then return 0 - n end
...   n
... end
>>> abs(-5)
5
```

## Credits

Mag is based on the Magpie language by [Robert Nystrom](http://stuffwithstuff.com/). His [blog posts](http://journal.stuffwithstuff.com/category/magpie/) are the foundational inspiration for this project.

# License

Licensed under the MIT license.
