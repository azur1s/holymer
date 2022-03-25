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
- `deno` for running Typescript
- Rust (if you're going to build from source)
- (Optional) if you use Vim, you can get the syntax highlighting [here](https://github.com/azur1s/hazure.vim)

# Installing
Currently there is only a build script on linux:
```
curl -s https://raw.githubusercontent.com/azur1s/hazure/master/build.sh | bash -s
```
or if you want to build in debug mode:
```
curl -s https://raw.githubusercontent.com/azur1s/hazure/master/build.sh | bash -s d
```

# Contributing
Found a bug? Found a better way to do something? Make a pull request or tell me in the issues tab! Anything contributions helps :D

Steps to build:
1) Clone this repo `https://github.com/azur1s/hazure.git`
2) Build executable `cargo build`
3) Try running some examples! `path/to/executable compile path/to/file.hz`

# License
Hazure is licensed under both [MIT license](https://github.com/azur1s/hazure/blob/master/LICENSE-MIT) and [Apache License](https://github.com/azur1s/hazure/blob/master/LICENSE-APACHE)
