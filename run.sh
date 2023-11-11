./build.sh
qemu-system-riscv64 -machine virt -bios none -nographic -kernel kernel.elf -serial mon:stdio
