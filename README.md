# Hycron
Programming language

```
func main :: () -> Int = {
    let msg :: String = "Hello, World";
    puts(msg);
    return 1;
};
```

# TODO
- Compliation
- Optimization
- Use [chumsky](https://github.com/zesterer/chumsky) instead of [nom](https://github.com/Geal/nom) for parsing
- Error reporting (better with chumsky)