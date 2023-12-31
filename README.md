# KabutOS

RISCV kernel.

## Installing Dependencies

Note: This requires Ubuntu or Debian. If you're on another distro, or the
script does not work, check out the [Setting up Distrobox] section

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
gdb-multiarch target/riscv64imac-unknown-none-elf/debug/krabby
> target remote localhost:1234

# GDB commands
si/stepi # Step by instruction
s/step  # step
n/next  # Next line
break <label> # Break at label
continue # continue until breakpoint
```

## Setting up Distrobox

This not required for all distros, but is required for development on Fedora

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
