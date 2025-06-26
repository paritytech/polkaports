#!/bin/sh
set -ex

# Install dependencies.
brew install lld@19 llvm@19 automake autoconf libtool gnu-sed make

# Add new commands to PATH.
prefix=/opt/homebrew
for bin in \
	"$prefix"/Cellar/llvm@19/19.1.7/bin \
	"$prefix"/opt/lld@19/bin \
	"$prefix"/opt/gnu-sed/libexec/gnubin \
	"$prefix"/opt/make/libexec/gnubin; do
	if ! test -e "$bin"; then
		printf "Directory %s doesn't exist.\n" "$bin" >&2
		exit 1
	fi
	printf "%s\n" "$bin" >>"$GITHUB_PATH"
done
