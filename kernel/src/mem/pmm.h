#ifndef __PMM_H__
#define __PMM_H__

#include <stddef.h>
#include <stdint.h>

#define PAGE_SIZE 4096

#define B_TO_PAGES(b)   ((b) / PAGE_SIZE)
#define ALIGN_DOWN(addr, align) ((addr) & ~((align)-1))
#define ALIGN_UP(addr, align)	    (((addr) + (align)-1) & ~((align)-1))

#define IS_PAGE_ALIGNED(num)	    ((num % PAGE_SIZE) == 0)

#define BIT_TO_PAGE(bit)    ((size_t)bit * 0x1000)
#define PAGE_TO_BIT(page)   ((size_t)page / 0x1000)

typedef struct
{
    uint8_t	*map;
    size_t	size;
} BITMAP_t;

void bitmap_set_bit(BITMAP_t *bitmap, int bit);
void bitmap_clear_bit(BITMAP_t *bitmap, int bit);
uint8_t bitmap_get_bit(BITMAP_t *bitmap, int bit);

uintptr_t limine_virtual_addr_to_phys_addr(uintptr_t virt_addr);
uintptr_t phys_addr_to_limine_virtual_addr(uintptr_t phys_addr);
uintptr_t phys_addr_to_limine_virtual_code_addr(uintptr_t phys_addr);


void pmm_init();
void free_pages(void *pointer, size_t page_count);
void *alloc_page(size_t page_count);


#endif // __PMM_H__