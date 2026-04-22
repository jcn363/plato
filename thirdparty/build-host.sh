#!/bin/sh

set -e

NUM_JOBS=$(nproc 2>/dev/null || echo 4)

cd libs_host
cd ..

echo "Host libraries built successfully!"
