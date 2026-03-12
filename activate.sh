#!/bin/sh
export COREVM_SYSROOT="$PWD"/sysroot
export COREVM_CC="${COREVM_CC:-clang}"
export COREVM_CXX="${COREVM_CXX:-clang++}"
export COREVM_LLD="${COREVM_LLD:-lld}"

# Prepend to PATH.
case ":$PATH:" in
*:"$COREVM_SYSROOT"/bin:*) ;;
*)
	export PATH="$COREVM_SYSROOT"/bin:"$PATH"
	;;
esac
