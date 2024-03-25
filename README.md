# KabutOS

RISC-V operating system

## Crates

* krabby - The kernel
* krabby-abi - ABI between userspace and kernel, and common types
* userspace/kanto - Userland library
* userspace/dratinit - The init program (process 1)
* userspace/gary - Userspace test suite
* embedded-line-edit - No-std manual line editing library
* crusty-line - No-std readline functionality
* page-alloc - Page-grain allocation library

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
./debug.sh

# GDB commands
si/stepi # Step by instruction
s/step  # step
n/next  # Next line
break <label> # Break at label
continue # continue until breakpoint
```

## Setting up Distrobox

While KabutOS will build anywhere, a GDB build for RISC-V is lacking in the
Fedora system packages, so installing Debian via Distrobox is useful for
debugging.

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
