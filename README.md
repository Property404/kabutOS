# KabutOS

RISCV kernel.

## Install Required Dependencies (Ubuntu)

```bash
sudo apt install \
    gcc-riscv64-unknown-elf \
    binutils-riscv64-unknown-elf \
    qemu-system-riscv64 \
    gdb-multiarch \
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

## Debugging

```
make virt-debug

# In another terminal
gdb-multiarch kernel.elf
> target remote localhost:1234

# GDB commands
si/stepi # Step by instruction
s/step  # step
n/next  # Next line
break <label> # Break at label
continue # continue until breakpoint
```
