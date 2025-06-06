#!/bin/sh
set -ex
for tool in "$CC" "$LD" "$AR" "$RANLIB"; do
	which "$tool"
	"$tool" --version
done
./setup.sh
