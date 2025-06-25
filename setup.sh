#!/bin/bash

CC="${CC:-clang}"
CXX="${CXX:-clang++}"
AR="${AR:-llvm-ar}"
RANLIB="${RANLIB:-llvm-ranlib}"

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
	mkdir -p src/malloc/mallocng
	run env \
		CFLAGS="-Wno-shift-op-parentheses -Wno-unused-command-line-argument -fpic -fPIE -mrelax --target=riscv64-unknown-none-elf -march=rv64emac_zbb_xtheadcondmov -mabi=lp64e -ggdb" \
		CC="$CC" \
		AR="$AR" \
		RANLIB="$RANLIB" \
		LIBCC="$PWD"/libclang_rt.builtins-riscv64.a \
		LDFLAGS="-Wl,--emit-relocs -Wl,--no-relax" \
		./configure \
		--prefix="$sysroot" \
		--target=riscv64 \
		--enable-wrapper=clang \
		--disable-shared
	run make clean
	run make -j
	run make install
}

musl_install() {
	# Install RISCV Linux headers.
	cp -r "$root"/sdk/riscv64-include/* "$sysroot"/include/
	# Install CoreVM-specific headers.
	case "$suffix" in
	polkavm) ;;
	corevm) ln -f "$root"/sdk/corevm_guest.h "$sysroot"/include/ ;;
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
suffix=
for x in "\$@"; do
	case "\$x" in
	-nostdlib) suffix=-nostdlib ;;
	*) ;;
	esac
done
exec "$CC" --config=$sysroot/clang\$suffix.cfg "\$@"
EOF
	chmod +x "$sysroot"/bin/polkavm-cc
	cat >"$sysroot"/bin/polkavm-c++ <<EOF
#!/bin/sh
suffix=
for x in "\$@"; do
	case "\$x" in
	-nostdlib) suffix=-nostdlib ;;
	*) ;;
	esac
done
exec "$CXX" --config=$sysroot/clang\$suffix.cfg "\$@"
EOF
	chmod +x "$sysroot"/bin/polkavm-c++
	ln -f "$root"/sdk/clang.cfg "$sysroot"/
	ln -f "$root"/sdk/clang-nostdlib.cfg "$sysroot"/
	# clang-18 and clang-19 on Ubuntu want libgcc
	# clang-20 on Fedora wants libgcc_s
	# busybox wants libgcc_eh
	mkdir -p "$sysroot"/lib
	for name in libgcc_s libgcc libgcc_eh; do
		touch "$sysroot"/lib/"$name".a
	done
}

main() {
	PS4='$0:$LINENO: 🏗️  ' set -ex
	root="$PWD"
	workdir="$(mktemp -d)"
	trap cleanup EXIT
	for suffix in polkavm corevm; do
		sysroot="$root"/sysroot-"$suffix"
		sysroot_init
		polkatool_install
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
