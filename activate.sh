#!/bin/sh
export COREVM_SYSROOT="$PWD"/sysroot-corevm
export COREVM_CC="${COREVM_CC:-clang}"
export COREVM_CXX="${COREVM_CXX:-clang++}"
export COREVM_LLD="${COREVM_LLD:-lld}"
export PATH="$COREVM_SYSROOT"/bin:"$PATH"
