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
}

build_quake() {
	cd "$root"/apps/quake
	make clean
	make -j
}

build_busybox() {
	cd "$root"/apps/busybox
	./build.sh
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
