#!/bin/sh

suffix="$1"
case "$suffix" in
polkavm | corevm) ;;
*)
	printf "usage: . ./activate.sh corevm|polkavm\n" >&2
	return 1
	;;
esac

export POLKAPORTS_SUFFIX="$suffix"
export POLKAPORTS_SYSROOT="$PWD"/sysroot-"$suffix"
export PATH="$POLKAPORTS_SYSROOT"/bin:"$PATH"
