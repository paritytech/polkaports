#!/bin/sh
set -ex
suffix="$1"
. ./activate.sh "$suffix"
cd apps/quake
make clean
make -j
