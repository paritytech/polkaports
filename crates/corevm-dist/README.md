This CLI tool downloads and installs binary releases of CoreVM build tools from
[PolkaPorts](https://github.com/paritytech/polkaports) repository.

## Installation

```bash
cargo install corevm-dist

# This command downloads and installs pre-built CoreVM tools into `~/.corevm`.
corevm-dist install

# Now you can build some code for CoreVM.
. ~/.corevm/env
git clone https://github.com/paritytech/polkaports
cd polkaports/apps/quake
make
```


## Self-update

Use the following command to update the tool itself and install new release of the tools.

```bash
corevm-dist update

# This is a shorthand for `cargo install corevm-dist && corevm-dist install`.
```
