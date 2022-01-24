# bobbylisp
another lisp dialect

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
Input(file) -> Parser -> Interpret(TODO)
  String       SExprs          IO
                     |->  Compile(TODO)
                              File 
```

## Installation
```bash
$ make
```
or
```bsah
$ make debug
```
The binary will be installed in `~/bin/blspc` run it with:
```
$ blspc -h
```

## Progress:
- [X] Lexer & Parser
- [ ] Syntax checker & Type checker
- [ ] Interpreter
- [ ] Compiler