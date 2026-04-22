#! /usr/bin/env bash

set -e

declare -a packages=()

# Use nproc for parallel building
NUM_JOBS=$(nproc 2>/dev/null || echo 4)

build_package() {
    local name="$1"
    (
        cd "$name"
        echo "Building ${name}..."
        [ -e kobo.patch ] && patch -p 1 < kobo.patch
        ./build-kobo.sh
    )
}

export -f build_package

echo "Building all packages..."
