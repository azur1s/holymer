# <img src="https://raw.githubusercontent.com/azur1s/bobbylisp/master/assets/icon.png" width="35"> bobbylisp
another lisp dialect
> Also available on https://git.ablecorp.us/azur/bobbylisp

## <img src="https://raw.githubusercontent.com/azur1s/bobbylisp/master/assets/icon.png" width="25"> Installation
```console
$ bash <(curl -s https://raw.githubusercontent.com/azur1s/bobbylisp/master/install.sh)
```
The binary will be installed in `~/bin/blspc` run it with:
```console
$ blspc help
```

### <img src="https://raw.githubusercontent.com/azur1s/bobbylisp/master/assets/icon.png" width="15"> Example
```console
$ blspc compile ./example/hello.blsp
$ blspc run ./hello.bsm
Hello, World!
```

## <img src="https://raw.githubusercontent.com/azur1s/bobbylisp/master/assets/icon.png" width="25"> Progress:
DONE:
- Parsing, Compiling, Running(VM)
- Intrinsic:
  - Function definition: `fun`
  - Variable definition: `let`
  - Do blocks: `do`
  - Printing: `print`
  - Condition: `if`
  - Math: 
    - `+` , `add`
    - `-` , `sub`
    - `*` , `mul`
    - `/` , `div`

TODO:
- Prove turing complete
- Do the intrinsic left
- Quote, Quasiquote, etc.
- Linter, for stuff like undefined variables, etc.
- Optimizing
- Remove unnecessary copying in the entire codebase (also with `.unwrap()`)
