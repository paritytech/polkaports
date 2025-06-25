#!/bin/sh

# A mirror of https://git.busybox.net/busybox that can keep up with the load.
url=https://github.com/igankevich/busybox
version=1_37_0
polkatool_args="--min-stack-size 65536"

CC="${CC:-clang}"
CXX="${CXX:-clang++}"
AR="${AR:-llvm-ar}"
AS="${AS:-llvm-as}"
NM="${NM:-llvm-nm}"
STRIP="${STRIP:-llvm-strip}"
OBJCOPY="${OBJCOPY:-llvm-objcopy}"
OBJDUMP="${OBJDUMP:-llvm-objdump}"

main() {
	set -ex
	workdir="$(mktemp -d)"
	trap cleanup EXIT
	root="$PWD"
	git clone --depth=1 --branch="$version" "$url" "$workdir"/busybox
	cd "$workdir"/busybox
	make defconfig
	# Override default configuration.
	sed -i \
		-e 's/.*\bCONFIG_STATIC\b.*/CONFIG_STATIC=y/g' \
		-e 's/.*\bCONFIG_SHA1_HWACCEL\b.*/CONFIG_SHA1_HWACCEL=n/g' \
		-e 's/.*\bCONFIG_SHA256_HWACCEL\b.*/CONFIG_SHA256_HWACCEL=n/g' \
		.config
	# Override tools.
	sed -i \
		-e 's|\bHOSTCC\s*=.*|HOSTCC = '"$CC"'|g' \
		-e 's|\bHOSTCXX\s*=.*|HOSTCXX = '"$CXX"'|g' \
		-e 's|\bAS\s*=.*|AS = '"$AS"'|g' \
		-e 's|\bCC\s*=.*|CC = polkavm-cc|g' \
		-e 's|\bAR\s*=.*|AR = '"$AR"'|g' \
		-e 's|\bNM\s*=.*|NM = '"$NM"'|g' \
		-e 's|\bSTRIP\s*=.*|STRIP = '"$STRIP"'|g' \
		-e 's|\bOBJCOPY\s*=.*|OBJCOPY = '"$OBJCOPY"'|g' \
		-e 's|\bOBJDUMP\s*=.*|OBJDUMP = '"$OBJDUMP"'|g' \
		-e 's|\bKBUILD_VERBOSE\s*=.*|KBUILD_VERBOSE = 1|g' \
		Makefile
	# Remove flags unsupported by clang.
	sed -i -e '/-fno-guess-branch-probability/d' Makefile.flags
	# Remove linker flags unsupported by polkatool.
	sed -i \
		-e 's/\bSORT_COMMON=.*/SORT_COMMON=/' \
		-e 's/\bSORT_SECTION=.*/SORT_SECTION=/' \
		-e 's/\bGC_SECTIONS=.*/GC_SECTIONS=/' \
		scripts/trylink
	# Override some definitions in the Makefile.
	cat >>Makefile.flags <<EOF
CPPFLAGS += -I$root/../../sdk/riscv64-include -D__linux__
CFLAGS += -flto
EOF
	make V=1 busybox
	polkatool link $polkatool_args busybox_unstripped -o busybox.corevm
	cp busybox.corevm "$root"/
}

cleanup() {
	rm -rf "$workdir"
}

main
