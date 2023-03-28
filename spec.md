# Specification

## Syntax

---

### Expressions

- Literals

    A literal is a value that is written directly
    into the source code.

    - Number

        An number literal is of type `f64` and can
        be expressed with or without a decimal point.
        - Examples: `1`, `3.14`, `.5`

        ```ebnf
        Number:
            Digits + (maybe '.' + Digits).
            (* Optional whole number, e.g. .5 *)
            ('.' + Digits).
        Digits:
            one or more of 0..9.
        ```

    - String

        A string literal can consist of zero or more
        characters enclosed in double quotes (`"`)
        - Examples: `"Hello, World"`,
                    `"They said \"Hi\""`,
                    `"Foo\nBar"`

        ```ebnf
        String:
            '"' + (zero or more of Character) + '"'.
        Character:
            any character except '"' or '\'.
            escape sequences.
        ```

    - Boolean

        A boolean literal can be either `true` or
        `false`.

        ```ebnf
        Boolean:
            'true' or 'false'.
        ```

    - Unit

        A unit literal is a value that represents
        the absence of a value.

        ```ebnf
        Unit:
            '()'.
        ```

- Identifiers

    An identifier is a name that is used to refer
    to a variable, function, or other entity.
    - Examples: `foo`, `barBaz`, `add2`

    ```ebnf
    Identifier:
        (Letter + zero or more of LetterOrDigit) but
        not any of Keywords.
    Letter:
        one of a..z or A..Z.
    LetterOrDigit:
        Letter or one of 0..9.
    ```

- Operators

    An operator is a symbol that is used to
    represent an operation.

    ```ebnf
    Binary:
        one of (
            (* Arithmetic *)
            + - * / %
            (* Comparison *)
            == != < <= > >=
            (* Logical *)
            && ||
        ).
    Unary:
        one of (- !).
    ```

- Application (Function Call)

    An application is an expression that calls a
    function with a list of arguments.
    It is not necessary that the callee is a
    function, but it must be an expression that
    evaluates to a function.

    ```ebnf
    Arguments:
        zero or more of Expression delimited by ','.
    Application:
        Expression + '(' + Arguments + ')'.
    ```

    - Examples:

        ```rust
        foo(1, 2, 3)
        (\x -> x + 1)(2)
        ```

- If-Else

    An if-else expression is an expression that
    evaluates to one of two expressions depending
    on the value of a condition.

    ```ebnf
    IfElse:
        'if' + Expression + 'then' + Expression + 'else' + Expression.
    ```

    - Examples:

        ```rust
        if true then 1 else 2
        if 1 == 2 then "foo" else "bar"
        ```

- Let Binding(s)

    There are 2 types of let bindings:
    - "Imperative" let bindings, which are
        similar to variable declarations in
        imperative languages (Javascript, Rust, etc.).

        ```ebnf
        Bindings:
            one or more of Binding delimited by ','.
        Let:
            'let' + Bindings.
        ```

        - Example:

          ```rust
          let x = 1 // -> ()
          x + 1     // -> 2
          ```

    - "Functional" let bindings, which are
        similar to variable declarations in
        functional languages (ML-family, etc.).

        ```ebnf
        LetIn:
            'let' + Bindings + 'in' + Expression.
        ```

        - Example:

            ```rust
            let x = 1, y = 2 in
                x + y // -> 3
            ```

- Block & Return

    A block is a sequence of expressions that are
    evaluated in order and the value of the last
    expression is returned (if not ended with a
    semicolon).

    A return expression is an expression that
    will exit the current block and return the
    value of the expression. It is not necessary
    to use a return expression in a block, but
    it could be useful for early termination.

    Any use of a return expression outside of a
    block is not allowed.

    ```ebnf
    Block:
        '{' + zero or more of Expression + '}'.
    Return:
        'return' + Expression.
    ```

    - Examples:

        ```rust
        {
            let x = 1;
            let y = 2;
            x + y
        }
        ```
        ```rust
        fun foo(): num = {
            if true then
                return 1;

            let bar = 42;
            bar
        };
        ```

### Keywords

Keywords are reserved words that cannot be
used as identifiers. They are used to
represent constructs of the language.

```ebnf
Keywords:
    if then else
    let fun return
```
