all: build
debug: build-debug

build:
	cd ./blspc; cargo build --release
	rm ~/bin/blspc -f
	mv ./target/release/blspc ~/bin/blspc

build-debug:
	cd ./blspc; cargo build
	rm ~/bin/blspc -f
	mv ./target/debug/blspc ~/bin/blspc
