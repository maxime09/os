#ifndef __PAGING_H__
#define __PAGING_H__

#include <stdint.h>


#define GB 0x40000000UL

// page table entry flags
#define PTE_PRESENT	    1
#define PTE_READ_WRITE	    2
#define PTE_USER_SUPERVISOR 4
#define PTE_WRITE_THROUGH   8
#define PTE_CHACHE_DISABLED 16
#define PTE_ACCESSED	    32
#define PTE_DIRTY	    64
#define PTE_PAT		    128
#define PTE_GLOBAL	    256

typedef uint64_t *PAGE_DIR;

void vmm_init(uintptr_t kernel_ro_start, uintptr_t kernel_ro_end, uintptr_t kernel_wr_start, uintptr_t kernel_wr_end);
void *find_phys_addr(PAGE_DIR pml4, uintptr_t virt_addr);
void map_page_kernel(uintptr_t phys_addr, uintptr_t virt_addr, int flags);
void map_page(PAGE_DIR current_page_directory, uintptr_t phys_addr, uintptr_t virt_addr, int flags);

#endif