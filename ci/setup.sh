#!/bin/sh
set -ex
"$CC" --version
"$LD" --version
"$AR" --version
"$RANLIB" --version
./setup.sh
