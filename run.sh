#!/usr/bin/env bash
# Run QEMU with the intent to debug

# Build Krabby
CARGO_OUTPUT=target/riscv64gc-unknown-none-elf/debug/krabby
cargo build

# Make a bin file because if we use the elf file QEMU will want to load it at
# the intended virtual address because it's stupid or something
riscv64-unknown-elf-objcopy -O binary ${CARGO_OUTPUT}{,.bin}

qemu-system-riscv64 -serial mon:stdio -nographic -machine virt -bios none ${@}  -kernel \
    "${CARGO_OUTPUT}.bin"
