# vy
another lisp dialect

## Installation
```console
$ bash <(curl -s https://raw.githubusercontent.com/azur1s/vy/master/install.sh)
```
The binary will be installed in `~/bin/vyc` run it with:
```console
$ vyc help
```

### Example
```console
$ vyc compile ./example/hello.vy
$ vyc run ./hello.bsm
Hello, World!
```

## Progress:
DONE:
- Parsing, Compiling, Running(VM)
- Intrinsic:
  - Function definition: `fun` (no arguments yet)
  - Variable definition: `def`
  - Do blocks: `do`
  - User input: `read`
  - Printing: `print`
  - Condition: `if`
  - Loops: `while`
  - Erroring: `throw`
  - Math: 
    - `+` , `add`
    - `-` , `sub`
    - `*` , `mul`
    - `/` , `div`
  - Comparison:
    - `=` , `equal`
    - `!` , `not`

TODO:
- Prove turing complete
- Do the intrinsic left
- Quote, Quasiquote, etc.
- Linter, for stuff like undefined variables, etc.
- Optimizing
- Remove unnecessary copying in the entire codebase (also with `.unwrap()`)
