#ifndef __RUST_EXPORT_H__
#define __RUST_EXPORT_H__

#include "io.h"

void kputs(char *s);
void kputc(char c);
void map_page_kernel(uintptr_t phys_addr, uintptr_t virt_addr, int flags);
void *alloc_page(size_t page_count);
void *alloc_page_phys_addr(size_t page_count);

#endif