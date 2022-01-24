all: build install
debug: build-debug install-debug

build:
	cargo build --release

install:
	rm -f ~/bin/blspc
	cp ./target/release/blspc ~/bin/blspc

build-debug:
	cargo build

install-debug:
	rm -f ~/bin/blspc
	cp ./target/debug/blspc ~/bin/blspc

install-lib:
	rm -rf ~/.bobbylib/
	mkdir -p ~/.bobbylib/
	cp -R ./lib/. ~/.bobbylib/