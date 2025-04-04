#!/bin/sh

set -euo pipefail

cargo install --root sysroot polkatool

cd libs/picoalloc
RUSTC_BOOTSTRAP=1 cargo build -p picoalloc_native --release --target=../../sdk/riscv64emac-unknown-none-polkavm.json -Zbuild-std=core,alloc
cd ../..

cd libs/musl
make -j
cd ../..

mkdir -p sysroot/bin
mkdir -p sysroot/lib
mkdir -p sysroot/tmp

ln -f sdk/polkavm-cc sysroot/bin
ln -f sdk/polkavm-c++ sysroot/bin
ln -f sdk/clang.cfg sysroot/bin

cd sysroot/tmp
ar x ../../libs/picoalloc/target/riscv64emac-unknown-none-polkavm/release/libpicoalloc_native.a
cd ../..

cp libs/musl/lib/*.a sysroot/lib
cp libs/musl/lib/*.o sysroot/lib
cp libs/musl/libclang_rt.builtins-riscv64.a sysroot/lib
ar r sysroot/lib/libc.a sysroot/tmp/picoalloc*.o

mkdir -p sysroot/include
cp -r libs/musl/include/* sysroot/include
cp -r libs/musl/arch/generic/* sysroot/include
cp -r libs/musl/arch/riscv64/* sysroot/include
cp -r libs/musl/obj/include/* sysroot/include

echo
echo "Setup finished!"
echo "Type 'source activate.sh' to activate the toolchain."
