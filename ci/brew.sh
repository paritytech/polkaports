#!/bin/sh
lld_prefix=/opt/homebrew/opt/lld@19
llvm_prefix=/opt/homebrew/Cellar/llvm@19/19.1.7
brew install lld@19 llvm@19 automake autoconf libtool
export PATH="$lld_prefix"/bin:"$llvm_prefix"/bin:"$PATH"
ls "$lld_prefix"/bin/* \
	"$llvm_prefix"/bin/* \
	/opt/homebrew/bin/*
