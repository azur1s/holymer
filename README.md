# <img src="https://raw.githubusercontent.com/azur1s/bobbylisp/master/assets/icon.png" width="35"> bobbylisp
another lisp dialect
> Also available on https://git.ablecorp.us/azur/bobbylisp

## <img src="https://raw.githubusercontent.com/azur1s/bobbylisp/master/assets/icon.png" width="25"> Installation
```bash
$ bash <(curl -s https://raw.githubusercontent.com/azur1s/bobbylisp/master/install.sh)
```
The binary will be installed in `~/bin/blspc` run it with:
```bash
$ blspc -h
```

### <img src="https://raw.githubusercontent.com/azur1s/bobbylisp/master/assets/icon.png" width="15"> Example
If no `-r` or `-c` specified. It will check for file extension instead.
If found `.blsp`, it will compile, if found `.bsm` it will run vm and interpret the bytecode.
```bash
$ blspc ./example/hello.blsp
$ blspc ./hello.bsm
Hello, World!
```

## <img src="https://raw.githubusercontent.com/azur1s/bobbylisp/master/assets/icon.png" width="25"> Progress:
DONE:
- Parsing, Compiling, Running(VM)
- Intrinsic:
  - `fun`, `do`, `print`
  - Math:

TODO:
- Do the intrinsic left
- Quote, Quasiquote, etc.
- Optimizing
- Remove unnecessary copying in the entire codebase
