#!/bin/sh
set -ex
for tool in "$CC" "$LLD" "$AR" "$RANLIB"; do
	which "$tool"
	"$tool" --version
done
./setup.sh
