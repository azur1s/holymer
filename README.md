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
- [ ] Compiler