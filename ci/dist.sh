#!/bin/sh

KERNEL="$(uname -s)"
ARCH="$(uname -m)"
SYSROOT_FILENAME="sysroot-$KERNEL-riscv64emac.tar.zst"
TOOLS_FILENAME="tools-$KERNEL-$ARCH.tar.zst"

main() {
	set -ex
	workdir="$(mktemp -d)"
	trap cleanup EXIT
	root="$PWD"
	set_permissions
	create_sysroot_archive
	create_tools_archive
	case "$kernel" in
	Linux) b2sum "$SYSROOT_FILENAME" "$TOOLS_FILENAME" ;;
	*) b2sum "$TOOLS_FILENAME" ;;
	esac
}

set_permissions() {
	# Reproducible permissions.
	find sysroot -type f -exec chmod 0644 \{\} \;
	find sysroot -type l -exec chmod --no-dereference 0777 \{\} \;
	find bin -type f -exec chmod 0755 \{\} \;
	find sysroot/* -type d -exec chmod 0755 \{\} \;
}

create_sysroot_archive() {
	if test "$KERNEL" != Linux; then
		# System root shouldn't depend on host OS.
		return
	fi
	# Resulting archive should be reproducible between CI job runs.
	cd sysroot
	{
		# We only care about non-hidden files, directories and symbolic links.
		find . -type f -not -name '.*' -print0
		find . -type d -not -name '.*' -print0
		find . -type l -not -name '.*' -print0
	} | env LC_ALL=C sort --unique --zero-terminated >"$workdir"/files
	create_tar_archive "$workdir"/files "$root"/"$SYSROOT_FILENAME"
	cd "$root"
}

create_tools_archive() {
	# Resulting archive should be reproducible between CI job runs.
	cd bin
	{
		# We only care about non-hidden files, directories and symbolic links.
		find . -type f -not -name '.*' -print0
		find . -type d -not -name '.*' -print0
		find . -type l -not -name '.*' -print0
	} | env LC_ALL=C sort --unique --zero-terminated >"$workdir"/files
	create_tar_archive "$workdir"/files "$root"/"$TOOLS_FILENAME"
	cd "$root"
}

create_tar_archive() {
	rm -f "$2"
	tar --create \
		--mtime=@0 \
		--numeric-owner \
		--owner=0 \
		--group=0 \
		--hard-dereference \
		--null \
		--files-from="$1" \
		--file=- |
		zstd --quiet --compress -10 -o "$2"
}

cleanup() {
	rm -rf "$workdir"
}

main
