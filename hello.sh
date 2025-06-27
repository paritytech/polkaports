#!/bin/sh
set -ex
if test -z ${POLKAPORTS_SUFFIX+x} || test -z ${POLKAPORTS_SYSROOT+x}; then
	printf "Run . ./activate.sh corevm|polkavm first.\n" >&2
	exit 1
fi
toolchain=nightly-2024-11-01
suffix=corevm
target_name=riscv64emac-"$POLKAPORTS_SUFFIX"-linux-musl
export RUSTC_BOOTSTRAP=1
rm -rf target/riscv64emac-corevm-linux-musl
cargo +"$toolchain" \
	build \
	--quiet \
	--package hello \
	--target="$POLKAPORTS_SYSROOT"/"$target_name".json \
	-Zbuild-std=core,alloc,std,panic_abort \
	-Zbuild-std-features=panic_immediate_abort
polkatool link --min-stack-size 8388608 target/"$target_name"/debug/hello -o hello.corevm
