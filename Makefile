
CC=riscv64-unknown-linux-gnu-gcc
CFLAGS=-Wall -Wextra -pedantic -Wextra -O0 -g
CFLAGS+=-static -ffreestanding -nostdlib -fno-rtti -fno-exceptions
CFLAGS+=-march=rv64gc -mabi=lp64d
INCLUDES=
LINKER_SCRIPT=-Tsrc/arch/lds/virt.lds
RUST_TARGET=./target/riscv64gc-unknown-none-elf
LIBS=-L$(RUST_TARGET)
SOURCES_ASM=$(wildcard src/arch/*.S)
LIB=-lkernel -lgcc
OUT=kernel.elf

QEMU=qemu-system-riscv64
MACH=virt
CPU=rv64
CPUS=4
MEM=128M
DRIVE=hdd.dsk

all: debug

release:
	cargo build -r
	$(CC) $(CFLAGS) $(LINKER_SCRIPT) $(INCLUDES) -o $(OUT) $(SOURCES_ASM) $(LIBS)/release $(LIB)

debug:
	cargo build
	$(CC) $(CFLAGS) $(LINKER_SCRIPT) $(INCLUDES) -o $(OUT) $(SOURCES_ASM) $(LIBS)/debug $(LIB)
	
run:
	$(QEMU) -machine $(MACH) -cpu $(CPU) -smp $(CPUS) -m $(MEM)  -nographic -serial mon:stdio -bios none -kernel $(OUT) -drive if=none,format=raw,file=$(DRIVE),id=foo -device virtio-blk-device,scsi=off,drive=foo


.PHONY: clean
clean:
	cargo clean
	rm -f $(OUT)
