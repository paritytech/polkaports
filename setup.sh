#!/bin/bash

AR="${AR:-llvm-ar}"
CC="${CC:-clang}"
CXX="${CXX:-clang++}"

cleanup() {
	rm -rf "$workdir"
}

polkatool_install() {
	cargo install --quiet --root "$sysroot" polkatool
}

picoalloc_build() {
	suffix="$1"
	shift
	cd "$root"/libs/picoalloc
	rm -rf target
	RUSTC_BOOTSTRAP=1 cargo build \
		-Zbuild-std=core,alloc \
        --quiet \
		--package picoalloc_native \
		--release \
		--target="$root"/sdk/riscv64emac-unknown-none-polkavm.json \
		"$@"
	mv -v target/riscv64emac-unknown-none-polkavm/release/libpicoalloc_native.a \
		target/riscv64emac-unknown-none-polkavm/release/libpicoalloc_native"$suffix".a
}

musl_build() {
	cd "$root"/libs/musl
	make clean
	make -j
}

musl_install() {
	mkdir -p "$sysroot"/include
	cp -r "$root"/libs/musl/include/* "$sysroot"/include
	cp -r "$root"/libs/musl/arch/generic/* "$sysroot"/include
	cp -r "$root"/libs/musl/arch/riscv64/* "$sysroot"/include
	cp -r "$root"/libs/musl/obj/include/* "$sysroot"/include

	# Install CoreVM-specific headers.
	case "$suffix" in
	polkavm) ;;
	corevm) cp "$root"/sdk/corevm_guest.h "$sysroot"/include ;;
	esac

	mkdir -p "$sysroot"/lib
	cp "$root"/libs/musl/lib/*.a "$sysroot"/lib
	cp "$root"/libs/musl/lib/*.o "$sysroot"/lib

	# Repack libc with picoalloc.
	rm -rf "$workdir"/repack
	mkdir -p "$workdir"/repack
	cd "$workdir"/repack
	"$AR" x "$root"/libs/picoalloc/target/riscv64emac-unknown-none-polkavm/release/libpicoalloc_native"$suffix".a
	cp "$root"/libs/musl/lib/libc.a .
	"$AR" r libc.a picoalloc*.o
	# Overwrite libc.a in the sysroot
	cp libc.a "$sysroot"/lib

	for another_suffix in "" -riscv64; do
		ln -f \
			"$root"/libs/musl/libclang_rt.builtins-riscv64.a \
			"$sysroot"/lib/libclang_rt.builtins"$another_suffix".a
	done
}

sysroot_init() {
	rm -rf "$sysroot"/bin
	mkdir -p "$sysroot"/bin
	cat >"$sysroot"/bin/polkavm-cc <<EOF
#!/bin/sh
exec "$CC" --config=$sysroot/clang.cfg "\$@"
EOF
	chmod +x "$sysroot"/bin/polkavm-cc
	cat >"$sysroot"/bin/polkavm-c++ <<EOF
#!/bin/sh
exec "$CXX" --config=$sysroot/clang.cfg "\$@"
EOF
	chmod +x "$sysroot"/bin/polkavm-c++
	ln -f "$root"/sdk/clang.cfg "$sysroot"/
}

main() {
	PS4='$0:$LINENO: 🏗️  ' set -ex
	root="$PWD"
	workdir="$(mktemp -d)"
	for suffix in polkavm corevm; do
		sysroot="$root"/sysroot-"$suffix"
		polkatool_install
		sysroot_init
		case "$suffix" in
		polkavm) picoalloc_build polkavm ;;
		corevm) picoalloc_build corevm --features corevm ;;
		esac
		musl_build
		musl_install
	done
	cat <<'EOF'

Setup finished!

Type one of the following commands to activate the toolchain.

    . ./activate.sh corevm
    . ./activate.sh polkavm

EOF
}

main
