#!/usr/bin/env bash
sudo apt install \
    vim tmux git \
    gcc-riscv64-unknown-elf \
    binutils-riscv64-unknown-elf \
    qemu-system-riscv64 \
    gdb-multiarch \
    make
rustup target add riscv64imac-unknown-none-elf

