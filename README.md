# vl
another lisp dialect

```lisp
(fun factorial [x]
    (if (<= x 1)
        1
        (* x (factorial (- x 1)))))

(def times 7)
(do
    (print (factorial times)))
```

Compliation flow:
```
Input(file) -> Lexer -> Parser -> Interpret
  String       Token     Expr         IO
                              |-> Compile(TODO)
                                     File 
```

Progress:
- [X] Lexer & Parser
- [ ] Syntax checker & Type checker
- [X] Interpreter
- [ ] Compiler

Problems:
- Parser only detect the first error.
- Parser can't detect `(()))` syntax error.