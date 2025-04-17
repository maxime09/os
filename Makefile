PREFIX=${HOME}/opt/cross
TARGET=i686-elf
CC=${PREFIX}/bin/${TARGET}-gcc
AS=nasm -felf32
CC_ARGS= -std=gnu99 -ffreestanding -O2 -Wall -Wextra
LD_ARGS= -ffreestanding -O2 -nostdlib


all: build/myos.iso

build:
	mkdir build

build/boot.o: build src/boot/boot.asm
	$(AS) src/boot/boot.asm -o build/boot.o

build/kernel.o: build src/kernel/kernel.c
	$(CC) -c src/kernel/kernel.c -o build/kernel.o $(CC_ARGS)

build/print.o: build src/string/print.c
	$(CC) -c src/string/print.c -o build/print.o $(CC_ARGS)

build/gdt.o: build src/kernel/gdt.c
	$(CC) -c src/kernel/gdt.c -o build/gdt.o $(CC_ARGS)

build/gdt_asm.o: build src/kernel/gdt.asm
	$(AS) src/kernel/gdt.asm -o build/gdt_asm.o

build/myos.bin: build src/linker.ld build/boot.o build/kernel.o build/print.o build/gdt.o build/gdt_asm.o
	$(CC) -T src/linker.ld -o build/myos.bin $(LD_ARGS) build/boot.o build/kernel.o build/print.o build/gdt.o build/gdt_asm.o -lgcc

build/isodir/boot/grub: build
	mkdir -p build/isodir/boot/grub

build/myos.iso: build/isodir/boot/grub build/myos.bin src/grub.cfg
	cp build/myos.bin build/isodir/boot
	cp src/grub.cfg build/isodir/boot/grub
	grub-mkrescue -o build/myos.iso build/isodir

run: build/myos.iso
	qemu-system-i386 -cdrom build/myos.iso

clean: 
	rm -rf build