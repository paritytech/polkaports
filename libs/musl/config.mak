ARCH=riscv64
CROSS_COMPILE=riscv64-elf-
CFLAGS=-Wno-shift-op-parentheses -Wno-unused-command-line-argument -fpic -fPIE -mrelax --target=riscv64-unknown-none-elf -march=rv64emac_zbb_xtheadcondmov -mabi=lp64e -ggdb
CC=clang
LIBCC=
LDFLAGS=libclang_rt.builtins-riscv64.a -Wl,--emit-relocs -Wl,--no-relax
