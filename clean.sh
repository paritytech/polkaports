#!/bin/sh

set -euo pipefail

cd apps/hello-world ; make clean ; cd ../..
cd apps/quake ; make clean ; cd ../..
cd libs/musl ; make clean ; cd ../..
cd libs/picoalloc ; cargo clean ; cd ../..
rm -Rf sysroot
