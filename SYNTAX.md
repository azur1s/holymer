# Syntax
> This language is still in development, the syntax can be changed at anytime.

Hazure is a [free-form](https://en.wikipedia.org/wiki/Free-form_language) syntax style, so the indentation is purely for readability.

```sml
fun main: void = do
    @write("Hello, World!");
end;
```

is the same as

```sml
fun main: void = do @write("Hello, World!"); end;
```

Hazure is also [expression-oriented](https://en.wikipedia.org/wiki/Expression-oriented_programming_language) like OCaml. There are currently 10 expressions:

1) Comment
    ```
    -- Comment!
    -{ Block Comment }-
    ```
2) Values / Types
    ```sml
    1
    true
    "string"
    variable
    ```
3) Unary and Binary
    ```sml
    1 + 2
    !true
    ```
4) Call and Intrinsic. Intrinsic starts with a `@`
    ```sml
    @write("Hello")
    foo("bar")
    ```
5) Pipeline
    ```sml
    "Hello, World" |> @write(_)
    ```
6) Variable declaration
    ```sml
    let foo: int = 727;
    let bar: string = "Hi";
    let baz: boolean = true;
    ```
7) Function declaration
    ```sml
    fun foo: void = @write("void returns nothing");
    fun bar (x: int): int = x + 1;
    fun baz (x: int) (y: int): int = x + y;
    ```
8) If conditions
    ```sml
    let cond: bool = true;
    if cond then
        @write("True");
    else
        do
            @write("False");
        end;
    end;
    ```
9) Case matching
    ```sml
    case 1 + 1 of
        | 2 -> @write("Yes");
        | else @write("How?");
    end;
    ```
10) Do notation. It allows you to have multiple expression because something like right hand side of the function declaration `fun a: int = ...` can only have 1 expression. Do allows you to bypass that.
    ```sml
    do
        @write("Hello, World");
        foo(bar);
        let baz: int = 6 + 10;
    end;
    ```

Hazure isn't a scripting language, so, you will need a main function.

```sml
fun main: void = do
    @write("Hello, World");
    @write(34 + 35);
end;
```
