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
Input(file) -> Parser -> Compile(Bytecode) -> Interpret(blvm)
  String       SExprs        Bytecode               IO
                                          |->     Compile
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
```bash
$ blspc ./example/hello.blsp
$ blvm hello.bbb
Hello, World!
```

## Progress:
- [X] Lexer & Parser
- [ ] Syntax checker & Type checker
- [ ] Interpreter
- [ ] Compiler
