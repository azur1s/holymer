# bobbylisp
another lisp dialect
> Also available on https://git.ablecorp.us/azur/bobbylisp

## Installation
```bash
$ bash <(curl -s https://raw.githubusercontent.com/azur1s/bobbylisp/master/install.sh)
```
The binary will be installed in `~/bin/blspc` run it with:
```bash
$ blspc -h
```

### Example
If no `-r` or `-c` specified. It will check for file extension instead.
If found `.blsp`, it will compile, if found `.bsm` it will run vm and interpret the bytecode.
```bash
$ blspc ./example/hello.blsp
$ blspc ./hello.bsm
Hello, World!
```

## Progress:
- [X] Lexer & Parser
- [ ] Syntax checker & Type checker
- [ ] Interpreter
- [X] Compiler
