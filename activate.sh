#!/bin/sh

suffix="$1"
case "$suffix" in
polkavm) picoalloc_build polkavm ;;
corevm) picoalloc_build corevm --features corevm ;;
*)
	printf "usage: . ./activate.sh corevm|polkavm" >&2
	exit 1
	;;
esac

export PATH="$PWD"/sysroot-"$suffix"/bin:"$PATH"
export POLKAPORTS_SUFFIX="$suffix"
