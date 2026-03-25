#!/bin/sh

main() {
	set -ex
	. "$COREVM_HOME"/env
	root="$PWD"
	workdir="$(mktemp -d)"
	trap cleanup EXIT
	build_quake
	build_busybox
	build_rust_apps
	build_c_apps
	build_cxx_apps
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
	rust_target=riscv64emac-corevm-linux-musl
	rust_stack_size=8388608
	cd "$root"
	rm -rf target/riscv64emac-corevm-linux-musl
	for dir in apps/rust/hello; do
		cd "$root"/"$dir"
		package="$(basename "$dir")"
		run env RUSTC_BOOTSTRAP=1 \
			cargo build \
			--quiet \
			--target="$COREVM_HOME"/sysroot/etc/"$rust_target".json \
			-Zbuild-std=core,alloc,std,panic_abort \
			-Zbuild-std-features=panic_immediate_abort
		polkatool link --min-stack-size "$rust_stack_size" \
			target/"$rust_target"/debug/"$package" \
			-o "$workdir"/"$package".corevm
		jam-blob set-meta \
			--name "$package" \
			--version 0.1 \
			--license 'Apache-2.0' \
			--author 'Parity Technologies <admin@parity.io>' \
			"$workdir"/"$package".corevm
	done
}

build_c_apps() {
	cd "$root"/apps/c
	make
}

build_cxx_apps() {
	cd "$root"/apps/c++
	make
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
