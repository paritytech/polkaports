#!/bin/sh

main() {
	set -e
	if test -z "$VERSION"; then
		printf "Please set VERSION.\n" >&2
		exit 1
	fi
	workdir="$(mktemp -d)"
	trap cleanup EXIT
	crate_version="$(sed -rne 's/^[ ]*version[ ]*=[ ]*"(.*)"/\1/p' crates/corevm-dist/Cargo.toml)"
    ret=0
	if test "$crate_version" != "$VERSION"; then
		ret=1
		printf "Crate version \"%s\" doesn't match Git tag \"%s\"\n" "$crate_version" "$VERSION" >&2
	fi
    return "$ret"
}

cleanup() {
	rm -rf "$workdir"
}

main "$@"
