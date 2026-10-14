#!/bin/sh

main() {
	set -e
	workdir="$(mktemp -d)"
	trap cleanup EXIT
	b2sum *.tar.zst | env LC_ALL=C sort >"$workdir"/b2sum.txt.expected
	check_file crates/corevm-dist/b2sum.txt "$workdir"/b2sum.txt.expected
}

check_file() {
	actual="$1"
	expected="$2"
	ret=0
	if ! diff "$actual" "$expected" >"$workdir"/diff; then
		ret=1
		{
			printf "Wrong %s\n\nExpected contents:\n" "$actual"
			cat "$expected"
			printf "\nActual contents:\n"
			cat "$actual"
			printf "\nDifference:\n"
			cat "$workdir"/diff
			printf "\n"
		} >&2
	fi
	rm "$expected"
	return "$ret"
}

cleanup() {
	rm -rf "$workdir"
}

main "$@"
