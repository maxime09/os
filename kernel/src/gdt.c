#include "gdt.h"
#include <stdint.h>
#include "mem_function.h"

static gdt_entry_t gdt_entries[GDT_ENTRY_COUNT];
static TSS_t tss;

uint8_t compute_access(uint8_t DPL, uint8_t S, uint8_t E, uint8_t DC, uint8_t RW){
    uint8_t result = 0;
    result |= 1 << 7; // Present bit
    result |= 1 << 0; // Accessed bit
    result |= (DPL & 0x3) << 5;
    result |= (S & 0x1) << 4;
    result |= (E & 0x1) << 3;
    result |= (DC & 0x1) << 2;
    result |= (RW & 0x1) << 1;
    return result;
}

uint8_t compute_flags(uint8_t G, uint8_t DB, uint8_t L){
    uint8_t result = 0;
    result |= (G & 0x1) << 3;
    result |= (DB & 0x1) << 2;
    result |= (L & 0x1) << 1;
    return result;
}

void gdt_set_entry(int index, uint64_t base, uint32_t limit, uint8_t DPL, uint8_t S, uint8_t E, uint8_t DC, uint8_t RW, uint8_t G, uint8_t DB, uint8_t L) {
    uint8_t flags = compute_flags(G, DB, L);
    uint8_t access = compute_access(DPL, S, E, DC, RW);
    gdt_entries[index].base_low = (base & 0xFFFF);
    gdt_entries[index].base_middle = (base >> 16) & 0xFF;
    gdt_entries[index].base_high = (base >> 24) & 0xFF;
    gdt_entries[index].limit_low = (limit & 0xFFFF);
    gdt_entries[index].granularity = (limit >> 16) & 0x0F;
    gdt_entries[index].granularity |= (flags << 4) & 0xF0;
    gdt_entries[index].access = access;
}

void gdt_set_tss_entry(int first_index, uint64_t tss_base){
    uint64_t low_base = tss_base & 0xffffffff;
    uint64_t high_base = (tss_base >> 32) & 0xffffffff;
    uint32_t limit = sizeof(TSS_t) - 1;
    uint8_t flags = 0;
    uint8_t access = 0x89;
    gdt_entries[first_index].base_low = (low_base & 0xFFFF);
    gdt_entries[first_index].base_middle = (low_base >> 16) & 0xFF;
    gdt_entries[first_index].base_high = (low_base >> 24) & 0xFF;
    gdt_entries[first_index].limit_low = (limit & 0xFFFF);
    gdt_entries[first_index].granularity = (limit >> 16) & 0x0F;
    gdt_entries[first_index].granularity |= (flags << 4) & 0xF0;
    gdt_entries[first_index].access = access;

    gdt_entries[first_index+1].limit_low = (high_base & 0xFFFF);
    gdt_entries[first_index+1].base_low = (high_base >> 16) & 0xFFFF;
    gdt_entries[first_index+1].base_middle = 0;
    gdt_entries[first_index+1].base_high = 0;
    gdt_entries[first_index+1].granularity = 0;
    gdt_entries[first_index+1].access = 0;
}

void gdt_init() {
    gdt_set_entry(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0); // NULL segment
    gdt_set_entry(1, 0, 0xffff, 0, 1, 1, 0, 1, 0, 0, 0); // 16 bit code segment
    gdt_set_entry(2, 0, 0xffff, 0, 1, 0, 0, 1, 0, 0, 0); // 16 bit data segment
    gdt_set_entry(3, 0, 0xffffffff, 0, 1, 1, 0, 1, 1, 1, 0); // 32 bit code segment
    gdt_set_entry(4, 0, 0xffffffff, 0, 1, 0, 0, 1, 1, 1, 0); // 32 bit data segment
    gdt_set_entry(5, 0, 0xffffffff, 0, 1, 1, 0, 1, 1, 0, 1); // 64 bit kernel code segment
    gdt_set_entry(6, 0, 0xffffffff, 0, 1, 0, 0, 1, 1, 0, 1); // 64 bit kernel data segment
    gdt_set_entry(7, 0, 0xffffffff, 0, 1, 1, 0, 1, 1, 0, 1); // 64 bit user code segment
    gdt_set_entry(8, 0, 0xffffffff, 0, 1, 0, 0, 1, 1, 0, 1); // 64 bit user data segment
    
    memset(&tss, 0, sizeof(TSS_t));

    gdt_set_tss_entry(9, (uint64_t)&tss);

    gdt_ptr gdt_p;
    gdt_p.limit = (sizeof(gdt_entry_t) * GDT_ENTRY_COUNT) - 1;
    gdt_p.base = (uint64_t)&gdt_entries;

    
    gdt_load(&gdt_p);
}

