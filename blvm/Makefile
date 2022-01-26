all: build install
debug: build-debug install-debug

build:
	cargo build --release

install:
	rm -f ~/bin/blvm
	cp ./target/release/blvm ~/bin/blvm

build-debug:
	cargo build

install-debug:
	rm -f ~/bin/blvm
	cp ./target/debug/blvm ~/bin/blvm