#!/bin/sh

suffix="$1"
case "$suffix" in
polkavm | corevm) ;;
*)
	printf "usage: . ./activate.sh corevm|polkavm\n" >&2
	return 1
	;;
esac

export PATH="$PWD"/sysroot-"$suffix"/bin:"$PATH"
export POLKAPORTS_SUFFIX="$suffix"
