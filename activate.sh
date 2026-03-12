#!/bin/sh
export COREVM_SYSROOT="$PWD"/sysroot
# Prepend to PATH.
case ":$PATH:" in
*:"$COREVM_SYSROOT"/bin:*) ;;
*)
	export PATH="$COREVM_SYSROOT"/bin:"$PATH"
	;;
esac
