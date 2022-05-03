#!/usr/bin/env bash

# Exit if subprocess return non-zero exit code
set -e

# Log function
log () {
    echo -e "\033[0;32m[LOG]\033[0m $1"
}
err () {
    echo -e "\033[0;31m[ERR]\033[0m $1"
}

# This will always be true unless there is
# missing executable that we need to use
install_pass=true

# Check if $1 is installed
check_installed () {
    if ! command -v $1 -h &> /dev/null
    then
        err "$1 is not installed"
        if [ install_pass ]; then
            install_pass=false
        fi
    fi
}

check_installed cargo
check_installed git
check_installed deno # deno is required for running transpiled program

# If all of the above is installed
if [ ! install_pass ]; then
    exit 1
fi
log "Dependencies is installed. Cloning..."

rm -rf ~/.cache/hazure/build/
git clone https://github.com/azur1s/hazure.git ~/.cache/hazure/build/

cd ~/.cache/hazure/build/

if [[ $1 == *"d"* ]]; then
    log "Building in debug..."
    cargo build
    rm ~/bin/hzc -f
    mv ~/.cache/hazure/build/target/debug/hzc ~/bin/hzc
else
    log "Building..."
    cargo build --release
    rm ~/bin/hzc -f
    mv ~/.cache/hazure/build/target/release/hzc ~/bin/hzc
fi

log "Build done. Cleaning up..."

rm -rf ~/.cache/hazure/build/

log "Done."
hzc -v
