# Hazure
Programming language that compiles to Typescript!

```sml
fun main: void = do
    @write("Hello, World!");
end;
```
or with the pipeline operator:
```sml
fun main: void = do
    "Hello, World!\n"
    |> @write(_);
end;
```

Note: Everything in this project can be changed at anytime! (I'm still finding out what work best for lots of thing) if you have an idea, feel free to create an issues about it, or even create a PR! (I'd be very happy)

# Prerequistie
- `deno`
- Rust (if you're going to build from source)

# Contributing
Found a bug? Found a better way to do something? Make a pull request or tell me in the issues tab! Anything contributions helps :D

Steps to build:
1) Clone this repo `https://github.com/azur1s/hazure.git`
2) Build executable `cargo build`
3) Try running some examples! `path/to/executable compile path/to/file.hz`

# License
Hazure is licensed under both [MIT license](https://github.com/azur1s/hazure/blob/master/LICENSE-MIT) and [Apache License](https://github.com/azur1s/hazure/blob/master/LICENSE-APACHE)