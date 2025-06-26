#ifndef __GDT_H__
#define __GDT_H__
#include "stdint.h"

#define GDT_ENTRY_COUNT       11
#define CPU_MAX_COUNT   64

typedef struct {
 uint16_t limit;
 uint64_t base;
} __attribute__((packed)) gdt_ptr;


typedef struct gdt_entry_t {
 uint16_t limit_low;
 uint16_t base_low;
 uint8_t base_middle;
 uint8_t access;
 uint8_t granularity;
 uint8_t base_high;
} __attribute__((packed)) gdt_entry_t;

typedef struct TSS_t
{
    uint32_t	reserved0;
    uint64_t	rsp0;
    uint64_t	rsp1;
    uint64_t	rsp2;
    uint64_t	reserved1;
    uint64_t	ist1;
    uint64_t	ist2;
    uint64_t	ist3;
    uint64_t	ist4;
    uint64_t	ist5;
    uint64_t	ist6;
    uint64_t	ist7;
    uint64_t	reserved2;
    uint16_t	reserved3;
    uint16_t	iopb_offset;
} __attribute__((packed)) TSS_t;

extern void gdt_load(gdt_ptr* gdt_descriptor);

void gdt_init();


#endif