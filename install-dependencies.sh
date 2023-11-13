#!/usr/bin/env bash
sudo apt-get install \
    vim tmux git \
    gcc-riscv64-unknown-elf \
    binutils-riscv64-unknown-elf \
    qemu-system-riscv64 \
    gdb-multiarch
rustup target add riscv64imac-unknown-none-elf

