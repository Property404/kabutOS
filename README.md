# KabutOS

RISCV kernel.

## Install Required Dependencies (Debian 12)

Install Distrobox:

```bash
# Fedora
sudo dnf install distrobox

# Ubuntu/Debian
sudo apt install distrobox
```

Install Debian 12 with distrobox:

```bash
distrobox-create --name kdev --image debian:12
distrobox enter kdev
```

Install `rustup` if not already installed:

```bash
curl --proto '=https' --tlsv1.3 -sSf https://sh.rustup.rs | sh
```

Install dependencies

```bash
./install-dependencies
```

## Building

```bash
cargo build
```

## Running on QEMU

```bash
cargo run
```

To exit QEMU, type `Ctrl-A` then `X`

## Debugging

```
cargo run -- -S -s

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
