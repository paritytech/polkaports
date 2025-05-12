#!/bin/sh

set -euo pipefail

cd libs/picoalloc
RUSTC_BOOTSTRAP=1 cargo build -p picoalloc_native --release --target=../../sdk/riscv64emac-unknown-none-polkavm.json -Zbuild-std=core,alloc
cd ../..

cd libs/musl
make -j
cd ../..

cd quake
make -j
cd ..
