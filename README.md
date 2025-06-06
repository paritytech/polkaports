# PolkaVM/CoreVM SDK

This repository contains the source code for `musl` library patched for PolkaVM/CoreVM.
It is built for RISCV,
uses `picoalloc` as memory allocator, and
forwards all system calls via `pvm_syscall` host-call.

Besides that we provide `polkavm-cc` and `polkavm-c++` wrappers
to build applications that use the `musl` port.


## How to build the SDK

```bash

# Build the toolchain for `polkavm` or `corevm`.
# Tested with `clang-19` and `clang-20`.
env CC=clang CXX=clang++ LD=lld ./setup.sh corevm

# Activate (setup environment variables) for the toolchain.
# Either `polkavm` or `corevm`.
. ./activate.sh corevm
```


## How to build an application using the SDK

```bash
cd apps/quake
make -j
```
