#include "paging.h"
#include "pmm.h"
#include "../mem_function.h"
#include "../kernel.h"
#include <stddef.h>
#include <stdint.h>

PAGE_DIR create_page_directory(){
    PAGE_DIR page_directory = alloc_page(1);
    uint64_t phys_addr = limine_virtual_addr_to_phys_addr(page_directory);
    map_page_current(phys_addr, page_directory, PTE_PRESENT | PTE_READ_WRITE);

    memset((void *) page_directory, 0, PAGE_SIZE);

    return page_directory;
}

static void hcf(void) {
    for (;;) {
        asm ("hlt");
    }
}

PAGE_DIR get_pml_entry(PAGE_DIR pml, uintptr_t index, int flags){
    if(pml[index] & 1){
        //entry exist
        // & ~(511) => remove bits 0 to 8;
        return (PAGE_DIR)(pml[index] & ~(511));
    }else{
        uint64_t page = (uint64_t)alloc_page(1);
        uint64_t phys_addr = limine_virtual_addr_to_phys_addr(page);
        map_page_current(phys_addr, page, PTE_PRESENT | PTE_READ_WRITE);
        memset((void *) page, 0, PAGE_SIZE);
        pml[index] = limine_virtual_addr_to_phys_addr((uint64_t)page) | flags;
        return (PAGE_DIR)(pml[index] & ~(511));
    }
}


void flush_tlb(void* addr){
    __asm__ volatile("invlpg (%0)" : : "r" (addr));
}

uintptr_t get_cr3(){
    uintptr_t res;
    __asm__ volatile("mov %%cr3, %0" : "=r"(res) : );
    return res;
}

void map_page_kernel(uintptr_t phys_addr, uintptr_t virt_addr, int flags){
    PAGE_DIR current_page_directory = (PAGE_DIR) phys_addr_to_limine_virtual_addr(get_cr3());
    map_page(current_page_directory, phys_addr, virt_addr, flags);
}

void map_page(PAGE_DIR current_page_directory, uintptr_t phys_addr, uintptr_t virt_addr, int flags){
    uintptr_t index4 = (virt_addr & ((uintptr_t)0x1ff << 39)) >> 39;
    uintptr_t index3 = (virt_addr & ((uintptr_t)0x1ff << 30)) >> 30;
    uintptr_t index2 = (virt_addr & ((uintptr_t)0x1ff << 21)) >> 21;
    uintptr_t index1 = (virt_addr & ((uintptr_t)0x1ff << 12)) >> 12;
    

    PAGE_DIR pml4 = current_page_directory;
    PAGE_DIR pml3 = (PAGE_DIR)phys_addr_to_limine_virtual_addr((uintptr_t)get_pml_entry(pml4, index4, flags));
    PAGE_DIR pml2 = (PAGE_DIR)phys_addr_to_limine_virtual_addr((uintptr_t)get_pml_entry(pml3, index3, flags));
    uintptr_t temp = (uintptr_t)get_pml_entry(pml2, index2, flags);
    PAGE_DIR pml1 = (PAGE_DIR)phys_addr_to_limine_virtual_addr(temp);

    pml1[index1] = phys_addr | flags;
    if(flags & PTE_READ_WRITE){
        // set read/write to higher level of pml
        pml2[index2] |= PTE_READ_WRITE;
        pml3[index3] |= PTE_READ_WRITE;
        pml4[index4] |= PTE_READ_WRITE;
    }
    flush_tlb((void *)virt_addr);
}


void map_page_current(uintptr_t phys_addr, uintptr_t virt_addr, int flags){
    PAGE_DIR page_directory = (PAGE_DIR)phys_addr_to_limine_virtual_addr(get_cr3());
    map_page(page_directory, phys_addr, virt_addr, flags);
}

void update_cr3(PAGE_DIR current_page_directory){
    uintptr_t addr = (uintptr_t)current_page_directory;
    __asm__ volatile("mov %0, %%cr3" : : "r" (addr) : "memory");
}

PAGE_DIR new_cr3_value;

uintptr_t get_rip(){
    uintptr_t res;
    __asm__ volatile("leaq (%%rip), %0" : "=r"(res) : );
    return res;
}

extern uint8_t limine_start;
extern uint8_t limine_end;

void vmm_init(uintptr_t kernel_ro_start, uintptr_t kernel_ro_end, uintptr_t kernel_wr_start, uintptr_t kernel_wr_end, uintptr_t initrd_start, uintptr_t initrd_end){
    PAGE_DIR root_page_directory = create_page_directory();
    kprintf("Starting to map virtual memory 0/5\n");

    // Identity map the first 4 GB
    for (uintptr_t i = 0; i < 4 * GB; i += PAGE_SIZE){
        map_page(root_page_directory, i, i, PTE_PRESENT | PTE_READ_WRITE);
        
    }
    kprintf("Mapping virtual memory 1/5\n");

    //map initrd
    for (uintptr_t i = ALIGN_DOWN(initrd_start, PAGE_SIZE); i < ALIGN_UP(initrd_end, PAGE_SIZE); i += PAGE_SIZE){
        map_page(root_page_directory, limine_virtual_addr_to_phys_addr(i), i, PTE_PRESENT | PTE_READ_WRITE);
    }
    kprintf("Mapping virtual memory 2/5\n");

    // Map kernel address space
    for (uintptr_t i = 0; i < 4 * GB; i += PAGE_SIZE){
        
        map_page(root_page_directory, i, phys_addr_to_limine_virtual_addr(i), PTE_PRESENT | PTE_READ_WRITE);
    }

    PAGE_DIR old_root_page_directory = (PAGE_DIR)phys_addr_to_limine_virtual_addr(get_cr3());


    uintptr_t limine_start_addr = (uintptr_t) &limine_start;
    uintptr_t limine_end_addr = (uintptr_t) &limine_end;

    kprintf("Mapping virtual memory 3/5\n");
    // map limine request section
    for(uintptr_t virt_addr = ALIGN_DOWN(limine_start_addr, PAGE_SIZE); virt_addr < limine_end_addr; virt_addr += PAGE_SIZE){
        uintptr_t phys_addr = (uintptr_t)find_phys_addr(old_root_page_directory, virt_addr);
        map_page(root_page_directory, phys_addr, virt_addr, PTE_PRESENT | PTE_READ_WRITE);
    }

    kprintf("Mapping virtual memory 4/5\n");
    // map kernel code section
    for(uintptr_t virt_addr = ALIGN_DOWN(kernel_ro_start, PAGE_SIZE); virt_addr < kernel_ro_end; virt_addr += PAGE_SIZE)
    {
        uintptr_t phys_addr = (uintptr_t)find_phys_addr(old_root_page_directory, virt_addr);
        map_page(root_page_directory, phys_addr, virt_addr, PTE_PRESENT | PTE_READ_WRITE);
    }

    kprintf("Mapping virtual memory 5/5\n");
    

    // map kernel data section
    for(uintptr_t virt_addr = ALIGN_DOWN(kernel_wr_start, PAGE_SIZE); virt_addr < kernel_wr_end; virt_addr += PAGE_SIZE)
    {
        uintptr_t phys_addr = (uintptr_t)find_phys_addr(old_root_page_directory, virt_addr);
        map_page(root_page_directory, phys_addr, virt_addr, PTE_PRESENT | PTE_READ_WRITE);
    }

    new_cr3_value = limine_virtual_addr_to_phys_addr(root_page_directory);
    
    update_cr3(new_cr3_value);

    kprintf("Paging setup.\n");
}

void slave_core_init_vmm(){
    update_cr3(new_cr3_value);
};

void *find_phys_addr(PAGE_DIR pml4, uintptr_t virt_addr){
    uintptr_t index4 = (virt_addr & ((uintptr_t)0x1ff << 39)) >> 39;
    uintptr_t index3 = (virt_addr & ((uintptr_t)0x1ff << 30)) >> 30;
    uintptr_t index2 = (virt_addr & ((uintptr_t)0x1ff << 21)) >> 21;
    uintptr_t index1 = (virt_addr & ((uintptr_t)0x1ff << 12)) >> 12;

    PAGE_DIR pml3 = (PAGE_DIR)phys_addr_to_limine_virtual_addr((uintptr_t)get_pml_entry(pml4, index4, 0));
    PAGE_DIR pml2 = (PAGE_DIR)phys_addr_to_limine_virtual_addr((uintptr_t)get_pml_entry(pml3, index3, 0));
    PAGE_DIR pml1 = (PAGE_DIR)phys_addr_to_limine_virtual_addr((uintptr_t)get_pml_entry(pml2, index2, 0));
    
    uintptr_t phys_addr = pml1[index1] & ~(2047);
    phys_addr |= (virt_addr) & 0xfff;
    // remove XD bit
    phys_addr &= ~(1ll<<63);

    return (void *) phys_addr;
}