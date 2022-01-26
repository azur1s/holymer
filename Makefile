all: build-blspc build-blvm

build-blspc:
	cd ./blspc; cargo build
	rm ~/bin/blspc -f
	mv ./target/debug/blspc ~/bin/blspc

build-blvm:
	cd ./blvm; cargo build
	rm ~/bin/blvm -f
	mv ./target/debug/blvm ~/bin/blvm
