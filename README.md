# MilkV-Duo-hal

**NOTE**: This is not a kv store. :P

## How To Use

- Install Rust toolchain
  - `rustup default nightly` - Set the default toolchain to nightly
  - `rustup target add riscv64imac-unknown-none-elf` - Add the RISC-V target
  - `cargo install cargo-binutils` - Install the `cargo-binutils` package
  - `rustup component add llvm-tools` - Install the `llvm-tools` component
- Python3 required

Steps for a blinky:

```console
> cargo objcopy --example blinky -- -O binary firmware.bin

> ./gen-fip.sh

> # copy fip.bin to your `boot` partition of your SD card

> # Insert the SD card into the board and power it on, watch serial 0 for output
```
