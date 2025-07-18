#!/bin/sh

main() {
	set -ex
	suffix="$1"
	. ./activate.sh "$suffix"
	root="$PWD"
	workdir="$(mktemp -d)"
	trap cleanup EXIT
	build_quake
	build_busybox
	build_rust_apps
}

build_quake() {
	cd "$root"/apps/quake
	run make clean
	run make -j
}

build_busybox() {
	cd "$root"/apps/busybox
	run ./build.sh
}

build_rust_apps() {
	rust_target=riscv64emac-"$suffix"-linux-musl
	rust_stack_size=8388608
    cd "$root"
    rm -rf target/riscv64emac-corevm-linux-musl
	for package in hello; do
		run env RUSTC_BOOTSTRAP=1 \
			cargo build \
			--quiet \
			--package "$package" \
			--target="$POLKAPORTS_SYSROOT"/"$rust_target".json \
			-Zbuild-std=core,alloc,std,panic_abort \
			-Zbuild-std-features=panic_immediate_abort
		polkatool link --strip --min-stack-size "$rust_stack_size" \
			target/"$rust_target"/debug/"$package" \
			-o "$workdir"/"$package"."$suffix"
	done
}

run() {
	set +e
	"$@" >"$workdir"/output 2>&1
	ret="$?"
	set -e
	if test "$ret" != 0; then
		cat "$workdir"/output >&2
		return 1
	fi
}

cleanup() {
	rm -rf "$workdir"
}

main "$@"
