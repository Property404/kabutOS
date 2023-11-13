QEMU=qemu-system-riscv64
EXECUTABLE_NAME=krabby/target/riscv64imac-unknown-none-elf/debug/krabby

QEMU_FLAGS=-kernel $(EXECUTABLE_NAME) -serial mon:stdio -nographic

ifeq ($(findstring -debug,$(MAKECMDGOALS)),-debug)
	QEMU_FLAGS+=-S -s
endif

all: $(EXECUTABLE_NAME)

$(EXECUTABLE_NAME):
	cd krabby && cargo build

lint:
	mdl $$(find . -name "*.md")

clean:
	cd krabby && cargo clean

# Run unit tests in qemu
# Eg: `make pi-test`
%-test:  % ;

# Way to go into debug mode for QEMU targets
# Example: `make pi-debug` or `make virt-debug`
%-debug: % ;

# QEMU targets
virt:
	$(QEMU) -machine virt -bios none $(QEMU_FLAGS)
