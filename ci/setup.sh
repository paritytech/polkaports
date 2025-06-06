#!/bin/sh
set -ex
clang --version
ld.lld --version
llvm-ar --version
./setup.sh
