# PolkaKernel

⚠️ This crate is work-in-progress. Expect API changes. ⚠️

This is Linux syscall abstraction layer for PolkaVM/CoreVM that translates Linux system calls to PolkaVM/CoreVM host-calls.
The host-calls are abstracted away with traits and can easily be replaced by any meaningful implementation.
For example, you can read/write files from/to memory instead of on-disk file system.

The main use case is to support running arbitrary programs written for Linux on PolkaVM/CoreVM without code rewrite.
To accomplish that one needs to build the program using PolkaPorts toolchain and
then run it via provided "player" tool (work-in-progress).
Up-to-date instructions are in the [PolkaPorts repository](https://github.com/paritytech/polkaports).

```text
╭────────────────╮   Host-calls   ╭─────────────╮   Linux system calls   ╭───────────────╮
│ PolkaVM/CoreVM │ ←────────────→ │ PolkaKernel │ ←────────────────────→ │ Guest program │
╰────────────────╯                ╰─────────────╯                        ╰───────────────╯
```
