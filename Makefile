UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
	CROSS_COMPILE?=riscv64-elf-
else
	CROSS_COMPILE?=riscv64-unknown-elf-
endif

LD=$(CROSS_COMPILE)ld
OBJCOPY=$(CROSS_COMPILE)objcopy
QEMU=qemu-system-riscv64
EXECUTABLE_NAME=kernel

LDFLAGS=-T linker.ld -g -nostdlib
QEMU_FLAGS=-kernel $(EXECUTABLE_NAME).elf -serial mon:stdio -nographic

OBJECTS=krabby/target/riscv64imac-unknown-none-elf/debug/libkrabby.a

ifeq ($(findstring -debug,$(MAKECMDGOALS)),-debug)
	QEMU_FLAGS+=-S -s
endif

all: $(EXECUTABLE_NAME).elf

$(EXECUTABLE_NAME).elf:
	cd krabby && cargo build
	$(LD) $(LDFLAGS) $(OBJECTS) -o $(EXECUTABLE_NAME).elf

$(EXECUTABLE_NAME).bin: $(EXECUTABLE_NAME).elf
	$(OBJCOPY) -O binary $< $@

lint:
	mdl $$(find . -name "*.md")

clean:
	cd krabby && cargo clean
	rm -f *.elf
	rm -f *.dtb
	rm -f *.bin
	rm -f *.processed

# Run unit tests in qemu
# Eg: `make pi-test`
%-test:  % ;

# Way to go into debug mode for QEMU targets
# Example: `make pi-debug` or `make virt-debug`
%-debug: % ;

# QEMU targets
virt: $(EXECUTABLE_NAME).bin
	$(QEMU) -machine virt -bios none $(QEMU_FLAGS)
