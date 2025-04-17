#include "gdt.h"
#include <stdint.h>

#define GDT_ENTRY_SIZE 8 

void encodeGDTEntry(u32 entry, u8 *target, u32 base, u32 limit, u8 access, u8 flags){

    u32 off = GDT_ENTRY_SIZE * entry;// offset;
 
    // Encode the limit
    target[off + 0] = limit & 0xff;
    target[off + 1] = (limit >> 8) & 0xff;
    target[off + 6] = (limit >> 16) & 0xff;

    //Encode the base
    target[off + 2] = base & 0xff;
    target[off + 3] = (base >> 8) & 0xff;
    target[off + 4] = (base >> 16) & 0xff;
    target[off + 7] = (base >> 24) & 0xff;

    // Encode the access byte
    target[off + 5] = access;

    // Encode the flags
    target[off + 6] |= (flags << 4);
}

u8 gdt[5*GDT_ENTRY_SIZE];

extern void load_gdt(u16 limit, u32 base);

void initgdt(){
    encodeGDTEntry(0, gdt, 0, 0xffffffff, 0x00, 0x0); // Null segment
    encodeGDTEntry(1, gdt, 0, 0xffffffff, 0x9A, 0xC); // Code segment
    encodeGDTEntry(2, gdt, 0, 0xffffffff, 0x92, 0xC); // Data segment
    encodeGDTEntry(3, gdt, 0, 0xffffffff, 0xFA, 0xC); // User code segment
    encodeGDTEntry(4, gdt, 0, 0xffffffff, 0xF2, 0xC); // User data segment

    load_gdt(5*GDT_ENTRY_SIZE - 1, (u32) &gdt);
}