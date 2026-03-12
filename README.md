# CoreVM SDK

This repository contains the source code for `musl` library patched for CoreVM.
It is built for RISCV,
uses `picoalloc` as memory allocator, and
forwards all system calls via `pvm_syscall` host-call to CoreVM service.

Besides that we provide `polkavm-cc`, `polkavm-c++`, `polkavm-lld` wrappers
to build applications that use the `musl` port.
Those wrappers require LLVM 20 toolchain to work properly.


## How to build the SDK

```bash

# Build the toolchain (needs LLVM 20 toolchain in the `PATH`).
./setup.sh

# Activate (setup environment variables) the toolchain.
. ./activate.sh
```


## How to build an application using the SDK

```bash
cd apps/quake
make -j
```


## How to run the application

```bash
jamt vm new quake.corevm 10000000000
corevm-builder SERVICE_ID
corevm-monitor SERVICE_ID
```
