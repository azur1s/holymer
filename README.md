# Hazure
Programming language that compiles to C++!

```sml
fun main: int = do
    @write("Hello, World!\n");
    return 69;
end;
```
or with the pipe operator:
```sml
fun main: int = do
    "Hello, World!\n"
    |> @write(_);
    return 69;
end;
```
> The `return 69` is the exit code (like C++), try running `echo $?` to see it!

Note: Everything in this project can be changed at anytime! (I'm still finding out what work best for lots of thing) if you have an idea, feel free to create an issues about it, or even create a PR! (I'd be very happy)

# Contributing
Found a bug? Found a better way to do something? Make a pull request or tell me in the issues tab! Anything contributions helps :D

Wanna see how it works under the hood? see the [How it works](https://github.com/azur1s/hazure#how-it-works) tab, you should probably understand it a bit more.

Steps to build:
1) Clone this repo `https://github.com/azur1s/hazure.git`
2) Run `sudo make build-lib` to build the library (for the transpiled output)
3) Build executable `cargo build`
4) Try running some examples! `path/to/executable compile path/to/file.hz`

# How it works
```
           Source (.hz)
              │ crates/main
              │
            Lexer produce Token
              │ crates/lexer
              │
           Parser produce AST
              │ crates/parser
              │
         Diagnostic(Parsing)
              │     │ crates/diagnostic
              │     ╰ Fail -> Print error -> Exit
             Pass
              │
              │
          Lowerer(?) produce HIR
              │ crates/hir
              │
          Type Checker (TODO)
              │   │
              │   ╰ Fail -> Print error -> Exit
             Pass
              │
              │
         Diagnostic(Lowering)
              │     │ crates/diagnostic
              │     ╰ Fail -> Print error -> Exit
             Pass
              │
              │
 Command   Codegen produce C++
 (spawn)      │ crates/codegen
    │         │
    │         │
 clang++ ─────┴───── Executable
(Command)
```

# Prerequistie
- `clang++`(preferred, default) or any C++ compiler
- `make` for Makefile
- Rust (if you're going to build from source)

# Problems
This is the problem(s) of the language I found throughout while developing it
- Diagnostics stuff only report one error, maybe because of the early return when errors make it only return after first error. Fix is maybe make an error type and continue doing it stuff even if found the error. (Fixable)

# Configuration
You can also configurate Hades compiler (currently you can only change the C++ compiler). Make a new file called `hades.toml` in the current working directory and the compiler will look for it! if there isn't one then it will use the default configuration:
```toml
[compiler]
compiler = "clang++"
```

# License
Hades is licensed under both [MIT license](https://github.com/azur1s/hades/blob/master/LICENSE-MIT) and [Apache License](https://github.com/azur1s/hades/blob/master/LICENSE-APACHE)