# KabutOS

RISCV kernel.

## Install Required Dependencies (Ubuntu)

```bash
sudo apt install \
    gcc-riscv64-unknown-elf \
    binutils-riscv64-unknown-elf \
    qemu-system-riscv64 \
    make
```

Fedora does not have the right toolchain, but you can use Ubuntu via `distrobox`

## Building

```bash
make
```

## Running on QEMU

```bash
make virt
```
