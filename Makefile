all: build
debug: build-debug

build:
	cd ./vyc; cargo build --release
	rm ~/bin/vyc -f
	mv ./target/release/vyc ~/bin/vyc

build-debug:
	cd ./vyc; cargo build
	rm ~/bin/vyc -f
	mv ./target/debug/vyc ~/bin/vyc
