#!/bin/bash

linux_tag=v6.15
linux_url=https://github.com/torvalds/linux
picoalloc_tag=v5.2.0
picoalloc_url=https://github.com/koute/picoalloc
polkatool_version=0.29.0
jam_program_blob_version=0.1.26
llvm_tag=llvmorg-22.1.0
llvm_url=https://github.com/llvm/llvm-project

CC=clang
CXX=clang++
LLD=lld
AR=llvm-ar
RANLIB=llvm-ranlib

riscv_cflags="--target=riscv64-unknown-none-elf -march=rv64emac_zbb_xtheadcondmov -mabi=lp64e -fpic -fPIE -mrelax"
riscv_ldflags="-Wl,--emit-relocs -Wl,--no-relax"

# Flags that improve reproducibility.
repro_cflags="-g0 -fno-ident"
repro_cxxflags="$repro_cflags"
repro_rustflags="-C debuginfo=0"

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
	env RUSTFLAGS="$repro_rustflags" \
		cargo install --quiet --root "$COREVM_HOME" "$@" polkatool@$polkatool_version
}

jam_program_blob_install() {
	env RUSTFLAGS="$repro_rustflags" \
		cargo install --quiet --root "$COREVM_HOME" "$@" jam-program-blob@$jam_program_blob_version
}

picoalloc_build() {
	git clone --depth=1 --branch="$picoalloc_tag" --quiet "$picoalloc_url" "$workdir"/picoalloc
	cd "$workdir"/picoalloc
	rm -rf target
	target_json="$("$COREVM_HOME"/bin/polkatool get-target-json-path)"
	RUSTC_BOOTSTRAP=1 RUSTFLAGS="$repro_rustflags" \
		cargo build \
		-Zbuild-std=core,alloc \
		--quiet \
		--package picoalloc_native \
		--release \
		--target="$target_json" \
		--features corevm
	mv -v target/riscv64emac-unknown-none-polkavm/release/libpicoalloc_native.a \
		libpicoalloc_native.a
}

musl_build() {
	cd "$root"/libs/musl
	mkdir -p src/malloc/mallocng
	run env \
		CFLAGS="$riscv_cflags -O3 $repro_cflags -ffile-prefix-map=$PWD=musl" \
		CC="$CC" \
		AR="$AR" \
		RANLIB="$RANLIB" \
		LIBCC="$PWD"/libclang_rt.builtins-riscv64.a \
		LDFLAGS="$riscv_ldflags" \
		./configure \
		--prefix="$sysroot" \
		--target=riscv64 \
		--disable-wrapper \
		--disable-shared
	run make clean
	run make -j
	run make install
}

musl_install() {
	# Install CoreVM-specific headers.
	ln -f "$root"/sdk/corevm_guest.h "$sysroot"/include/
	cp "$root"/libs/musl/arch/riscv64/polkavm_guest.h "$sysroot"/include/

	mkdir -p "$sysroot"/lib
	cp "$root"/libs/musl/lib/*.a "$sysroot"/lib
	cp "$root"/libs/musl/lib/*.o "$sysroot"/lib

	# Repack libc with picoalloc.
	rm -rf "$workdir"/repack
	mkdir -p "$workdir"/repack
	cd "$workdir"/repack
	"$AR" x "$workdir"/picoalloc/libpicoalloc_native.a
	cp "$root"/libs/musl/lib/libc.a .
	"$AR" r libc.a picoalloc*.o
	# Overwrite libc.a in the sysroot
	cp libc.a "$sysroot"/lib

	for suffix in "" -riscv64; do
		ln -f \
			"$root"/libs/musl/libclang_rt.builtins-riscv64.a \
			"$sysroot"/lib/libclang_rt.builtins"$suffix".a
	done
}

linux_install() {
	git clone --depth=1 --branch="$linux_tag" "$linux_url" "$workdir"/linux
	cd "$workdir"/linux
	run make headers_install ARCH=riscv CONFIG_ARCH_RV64I=y INSTALL_HDR_PATH="$sysroot"
	cd "$root"
}

sysroot_init() {
	rm -rf "$COREVM_HOME"/bin
	mkdir -p "$COREVM_HOME"/bin
	cat >"$COREVM_HOME"/bin/polkavm-cc <<'EOF'
#!/bin/sh
suffix=
for x in "$@"; do
	case "$x" in
	-nostdlib) suffix=-nostdlib ;;
	*) ;;
	esac
done
exec "${COREVM_CC:-clang}" --config="$COREVM_HOME"/sysroot/etc/clang$suffix.cfg "$@"
EOF
	chmod +x "$COREVM_HOME"/bin/polkavm-cc
	cat >"$COREVM_HOME"/bin/polkavm-c++ <<'EOF'
#!/bin/sh
suffix=
for x in "$@"; do
	case "$x" in
	-nostdlib) suffix=-nostdlib ;;
	*) ;;
	esac
done
exec "${COREVM_CXX:-clang++}" --config="$COREVM_HOME"/sysroot/etc/clang++$suffix.cfg "$@"
EOF
	chmod +x "$COREVM_HOME"/bin/polkavm-c++
	cat >"$COREVM_HOME"/bin/polkavm-lld <<'EOF'
#!/bin/sh
exec "${COREVM_LLD:-lld}" "$@" \
    --sysroot="$COREVM_HOME"/sysroot \
    -L"$COREVM_HOME"/sysroot/lib \
    "$COREVM_HOME"/sysroot/lib/Scrt1.o \
    "$COREVM_HOME"/sysroot/lib/crti.o \
    "$COREVM_HOME"/sysroot/lib/crtn.o
EOF
	chmod +x "$COREVM_HOME"/bin/polkavm-lld
	mkdir -p "$sysroot"/etc
	ln -f "$root"/sdk/clang.cfg "$sysroot"/etc/
	ln -f "$root"/sdk/clang-nostdlib.cfg "$sysroot"/etc/
	ln -f "$root"/sdk/clang++.cfg "$sysroot"/etc/
	ln -f "$root"/sdk/clang++-nostdlib.cfg "$sysroot"/etc/
	ln -f "$root"/sdk/riscv64emac-corevm-linux-musl.json "$sysroot"/etc/
	# clang-18 and clang-19 on Ubuntu want libgcc
	# clang-20 on Fedora wants libgcc_s
	# busybox wants libgcc_eh
	# rust wants libunwind
	mkdir -p "$sysroot"/lib
	for name in libgcc_s libgcc libgcc_eh libunwind; do
		touch "$sysroot"/lib/"$name".a
	done
	# CMake cross-compilation configuration.
	cat >"$sysroot"/toolchain.cmake <<'EOF'
set(CMAKE_SYSTEM_NAME Linux)
set(CMAKE_C_COMPILER $ENV{COREVM_HOME}/bin/polkavm-cc)
set(CMAKE_CXX_COMPILER $ENV{COREVM_HOME}/bin/polkavm-c++)
set(CMAKE_FIND_ROOT_PATH $ENV{COREVM_HOME}/sysroot)
set(CMAKE_SYSROOT $ENV{COREVM_HOME}/sysroot)
set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
# This is a hack to make cmake cross-compilation work on MacOS.
set(CMAKE_C_COMPILER_WORKS 1)
set(CMAKE_CXX_COMPILER_WORKS 1)
EOF
}

libcxx_install() {
	# Custom config just for libcxx.
	cat >"$workdir"/libcxx.cmake <<EOF
set(CMAKE_SYSTEM_NAME Linux)
set(CMAKE_C_COMPILER $CC)
set(CMAKE_CXX_COMPILER $CXX)
set(CMAKE_FIND_ROOT_PATH $sysroot)
set(CMAKE_SYSROOT $sysroot)
set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
# This is a hack to make cmake cross-compilation work on MacOS.
set(CMAKE_C_COMPILER_WORKS 1)
set(CMAKE_CXX_COMPILER_WORKS 1)
EOF
	git clone --depth=1 --branch="$llvm_tag" "$llvm_url" "$workdir"/llvm
	# Configure libcxx first.
	cd "$workdir"/llvm/libcxx
	# Fix script permissions.
	chmod +x utils/generate_iwyu_mapping.py
	# Remove existing headers from the sysroot.
	rm -rf $sysroot/include/c++
	rm -rf build
	mkdir build
	cd build
	run env \
		CXXFLAGS="$riscv_cflags --sysroot=$sysroot -I$sysroot/include/c++/v1 -D_GNU_SOURCE -O3 $repro_cxxflags -ffile-prefix-map=$workdir/llvm/libcxx=libcxx" \
		LDFLAGS="$riscv_ldflags -nostdlib" \
		cmake \
		-DCMAKE_BUILD_TYPE=Release \
		-DCMAKE_INSTALL_PREFIX="$sysroot" \
		-DCMAKE_TOOLCHAIN_FILE="$workdir"/libcxx.cmake \
		-DLIBCXX_ENABLE_STATIC=1 \
		-DLIBCXX_ENABLE_SHARED=0 \
		-DLIBCXX_ENABLE_EXCEPTIONS=0 \
		-DLIBCXX_ENABLE_RTTI=1 \
		-DLIBCXX_INCLUDE_TESTS=0 \
		-DLIBCXX_ENABLE_RANDOM_DEVICE=0 \
		-DLIBCXX_HAS_TERMINAL_AVAILABLE=0 \
		-DLIBCXX_ENABLE_THREADS=0 \
		-DLIBCXX_ENABLE_MONOTONIC_CLOCK=0 \
		-DLIBCXX_ENABLE_TIME_ZONE_DATABASE=0 \
		-DLIBCXX_INCLUDE_BENCHMARKS=0 \
		-DLIBCXX_INCLUDE_DOCS=0 \
		-DLIBCXX_USE_COMPILER_RT=1 \
		-DLIBCXX_ENABLE_STATIC_ABI_LIBRARY=1 \
		-DLIBCXX_HAS_MUSL_LIBC=1 \
		..
	# Configure libcxxabi.
	cd "$workdir"/llvm/libcxxabi
	rm -rf build
	mkdir build
	cd build
	run env \
		CXXFLAGS="$riscv_cflags -I$workdir/llvm/libcxx/build/include/c++/v1 -I$workdir/llvm/libcxx/include -D_GNU_SOURCE -O3 $repro_cxxflags -ffile-prefix-map=$workdir/llvm/libcxxabi=libcxxabi" \
		LDFLAGS="$riscv_ldflags -nostdlib" \
		cmake \
		-DCMAKE_BUILD_TYPE=Release \
		-DCMAKE_INSTALL_PREFIX="$sysroot" \
		-DCMAKE_VERBOSE_MAKEFILE=1 \
		-DCMAKE_TOOLCHAIN_FILE="$workdir"/libcxx.cmake \
		-DLIBCXXABI_ENABLE_EXCEPTIONS=0 \
		-DLIBCXXABI_USE_LLVM_UNWINDER=0 \
		-DLIBCXXABI_ENABLE_STATIC_UNWINDER=1 \
		-DLIBCXXABI_USE_COMPILER_RT=1 \
		-DLIBCXXABI_ENABLE_THREADS=0 \
		-DLIBCXXABI_HAS_PTHREAD_API=0 \
		-DLIBCXXABI_INCLUDE_TESTS=0 \
		-DLIBCXXABI_ENABLE_SHARED=0 \
		-DLIBCXXABI_ENABLE_STATIC=1 \
		-DLIBCXXABI_SILENT_TERMINATE=1 \
		..
	# Build libcxxabi.
	run make -j
	run make install
	# Build libcxx.
	cd "$workdir"/llvm/libcxx/build
	run make -j
	# Run the script manually (cmake doesn't run it for some reason).
	../utils/generate_iwyu_mapping.py -o include/c++/v1/libcxx.imp
	run make install
	cd "$root"
}

run_single() {
	case "$1" in
	musl)
		sysroot="$root"/sysroot
		picoalloc_build
		musl_build
		musl_install
		;;
	*)
		printf "Uknown subcommand: '%s'\n" "$1"
		return 1
		;;
	esac
}

main() {
	set -ex
	root="$PWD"
	export COREVM_HOME="${COREVM_HOME:-$HOME/.corevm}"
	workdir="$(mktemp -d)"
	trap cleanup EXIT
	if ! test -z ${1+x}; then
		run_single "$1"
		exit 0
	fi
	sysroot="$COREVM_HOME"/sysroot
	sysroot_init
	if test -n "$TOOLS_RUST_TARGET"; then
		polkatool_install --target "$TOOLS_RUST_TARGET"
		jam_program_blob_install --target "$TOOLS_RUST_TARGET"
	else
		polkatool_install
		jam_program_blob_install
	fi
	picoalloc_build
	musl_build
	musl_install
	linux_install
	libcxx_install
	rm -rf "$sysroot"/share/man
	cp crates/corevm-dist/env.sh "$COREVM_HOME"/env
	case "$COREVM_HOME" in
	"$HOME"/.corevm) dir="~/.corevm" ;;
	*) dir="$COREVM_HOME" ;;
	esac
	cat <<EOF

Setup finished!

Type the following command to activate the toolchain.

    . $dir/env

EOF
}

main "$@"
