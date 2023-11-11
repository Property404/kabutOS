./build.sh
qemu-system-riscv64 -machine virt -bios none -kernel kernel.elf -serial mon:stdio
