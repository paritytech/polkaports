#!/bin/sh
set -ex

# Install dependencies.
sudo -n apt-get -qq update
sudo -n apt-get -qq install -y clang-20 lld-20 llvm-20 autotools-dev xxd

# Add new commands to PATH.
bin=/usr/lib/llvm-20/bin
if ! test -e "$bin"; then
	printf "Directory %s doesn't exist.\n" "$bin" >&2
	exit 1
fi
printf "%s\n" "$bin" >>"$GITHUB_PATH"
