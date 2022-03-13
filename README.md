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

# How it works
```js
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

# Configuration
You can also configurate Hades compiler (currently you can only change the C++ compiler). Make a new file called `hades.toml` in the current working directory and the compiler will look for it! if there isn't one then it will use the default configuration:
```toml
[compiler]
compiler = "clang++"
```

# License
Hades is licensed under both [MIT license](https://github.com/azur1s/hades/blob/master/LICENSE-MIT) and [Apache License](https://github.com/azur1s/hades/blob/master/LICENSE-APACHE)