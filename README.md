# bobbylisp
another lisp dialect
> Also available on https://git.ablecorp.us/azur/bobbylisp

```lisp
; example/s.blsp
(fun factorial (x)
    (if (<= x 1)
        1
        (* x (factorial (- x 1)))))
(do
    (print (factorial 7)))
```

Compliation flow:
```
Input(file) -> Parser -> Compile(Bytecode) -> Interpret
  String       SExprs        Bytecode            IO
                                          |->  Compile
                                              Assembly(?)
```

## Installation
```bash
$ make
```
or
```bash
$ make debug
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
