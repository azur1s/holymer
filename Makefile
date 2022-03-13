build-debug:
	@echo "Building executable (debug)... done"
	cargo build
	cp ./target/debug/hazure ~/bin/hazure -r
	@echo "Building executable (debug)... done"

build-lib:
	@echo "Building lib..."
	rm -rf /usr/include/hazure/
	cp ./lib/. /usr/include/hazure/ -r
	@echo "Building lib... done"