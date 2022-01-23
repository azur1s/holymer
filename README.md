# vl
another lisp dialect

```lisp
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

Progress:
- [X] Lexer & Parser
- [ ] Syntax checker & Type checker
- [ ] Interpreter
- [ ] Compiler

Problems:
- Parser can't detect `(()))` syntax error.