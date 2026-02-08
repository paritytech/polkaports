# PolkaVM/CoreVM SDK

This repository contains the source code for `musl` library patched for PolkaVM/CoreVM.
It is built for RISCV,
uses `picoalloc` as memory allocator, and
forwards all system calls via `pvm_syscall` host-call.

Besides that we provide `polkavm-cc` and `polkavm-c++` wrappers
to build applications that use the `musl` port.


## How to build the SDK

```bash

# Build the toolchain for `polkavm` and `corevm`.
# Tested with `clang-19` and `clang-20`.
env CC=clang CXX=clang++ LLD=lld ./setup.sh

# Activate (setup environment variables) for the toolchain.
# Either `polkavm` or `corevm`.
. ./activate.sh corevm
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

## Default polkaport Building for PVM build compatibility.

`jam-pvm-builder` expect polkaports app to follow these rules:

- POLKAPORT_OUTFILE: file name for output.
- POLKAPORT_NAME: name in metadata, default value in `Makefile`.
- POLKAPORT_VERSION: version in metadata, default value in `Makefile`.
- POLKAPORT_LICENSE: license in metadata, default value in `Makefile`.
- POLKAPORT_AUTHOR: author in metadata, default value in `Makefile`.
- Release compile to `output/${POLKAPORT_OUTFILE}.core_vm` only by simply running `make -j`.
