#!/bin/sh

build_quake() {
	cd "$root"/apps/quake
	make clean
	make -j
}

build_busybox() {
	cd "$root"/apps/busybox
    ./build.sh
}

main() {
	set -ex
	suffix="$1"
	. ./activate.sh "$suffix"
	root="$PWD"
    build_quake
    build_busybox
}

main "$@"
