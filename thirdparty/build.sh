#! /usr/bin/env bash

set -e

declare -a packages=(zlib bzip2 libpng libjpeg openjpeg jbig2dec freetype2 harfbuzz gumbo djvulibre mupdf)

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

# Certain packages depend on others (e.g., harfbuzz depends on freetype, mupdf depends on many)
# So we still need some ordering, but we can parallelize the independent ones.

# Group 1: Independent
echo "Building independent packages..."
printf "%s\n" zlib bzip2 libpng libjpeg openjpeg jbig2dec gumbo djvulibre | xargs -I {} -P "$NUM_JOBS" bash -c "build_package {}"

# Group 2: Freetype (depends on zlib, etc. but they are built)
build_package freetype2

# Group 3: Harfbuzz (depends on freetype)
build_package harfbuzz

# Group 4: MuPDF (depends on almost everything)
build_package mupdf
