#!/bin/sh
set -ex

# Install dependencies.
brew install lld@19 llvm@19 automake autoconf libtool

# Add new commands to PATH.
llvm_prefix=/opt/homebrew/Cellar/llvm@19/19.1.7
lld_prefix=/opt/homebrew/opt/lld@19
for prefix in "$llvm_prefix" "$lld_prefix"; do
	if ! test -e "$prefix"/bin; then
		printf "Directory %s doesn't exist.\n" "$prefix"/bin >&2
		exit 1
	fi
	printf "%s\n" "$prefix"/bin >>"$GITHUB_PATH"
done
