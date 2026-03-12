#!/bin/bash

main() {
	set -ex
	dir1="$1"
	dir2="$2"
	if ! test -d "$dir1" || ! test -d "$dir2"; then
		exit 1
	fi
	set +e
	diff -r "$dir1" "$dir2"
	if test "$?" = 0; then
		exit 0
	fi
	set -e
	workdir="$(mktemp -d)"
	trap cleanup EXIT
	diff_recursive_binary
	exit 1
}

diff_recursive_binary() {
	set +x
	(cd "$dir1" && find . -type f) | env LC_ALL=C sort >"$workdir"/files1
	(cd "$dir2" && find . -type f) | env LC_ALL=C sort >"$workdir"/files2
	env LC_ALL=C comm -1 -2 "$workdir"/files1 "$workdir"/files2 >"$workdir"/common-files
	while read -r file; do
		diff <(xxd -d "$dir1"/"$file") <(xxd -d "$dir2"/"$file") >"$workdir"/diff || {
            printf "Diff %s\n" "$file" >&2
			cat "$workdir"/diff
		}
	done <"$workdir"/common-files
	set -x
}

cleanup() {
	rm -rf "$workdir"
}

main "$@"
