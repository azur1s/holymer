# Build the libraries that are used by generated C code.
build_lib:
	sudo mkdir -p /usr/include/hycron
	sudo cp -r ./lib/* /usr/include/hycron