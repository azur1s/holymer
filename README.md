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

Progress:
- [X] Lexer & Parser
- [ ] Syntax checker & Type checker
- [ ] Compiler

Problems:
- Parser only detect the first error and quit.
- Parser can't detect `(()))` syntax error.