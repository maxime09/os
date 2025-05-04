#ifndef __KERNEL_H__
#define __KERNEL_H__

#include <stdint.h>
#include <stddef.h>
#include "limine.h"

void kprintf(const char *format, ...);

extern struct limine_memmap_response *memmap;
extern size_t hhdm_offset;

#endif //__KERNEL_H__