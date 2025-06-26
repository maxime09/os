#include "pmm.h"
#include "../kernel.h"
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include "../mem_function.h"

uintptr_t limine_virtual_addr_to_phys_addr(uintptr_t virt_addr){
    return virt_addr - hhdm_offset;
}

uintptr_t phys_addr_to_limine_virtual_addr(uintptr_t phys_addr){
    return phys_addr + hhdm_offset;
}

uintptr_t phys_addr_to_limine_virtual_code_addr(uintptr_t phys_addr){
    return phys_addr + 0xFFFFFFFF80000000UL;
}


struct physical_mem_info{
    size_t memory_size;
    uint32_t max_pages;
    uint32_t used_pages;
};

struct physical_mem_info info;
BITMAP_t bitmap;

void pmm_init(){
    struct limine_memmap_entry *current_entry;

    size_t current_top;
    size_t top_addr = 0;
    
    // find highest usable page
    for(size_t i = 0; i < memmap->entry_count; i++){
        current_entry = memmap->entries[i];

        // skip non usable entries;
        if(current_entry->type != LIMINE_MEMMAP_USABLE && 
            current_entry->type != LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE)
            continue;

        current_top = current_entry->base + current_entry->length;

        if(current_top > top_addr)
            top_addr = current_top;
    }

    info.memory_size = top_addr;
    info.max_pages = B_TO_PAGES(info.memory_size);
    info.used_pages = info.max_pages;

    size_t bitmap_size = ALIGN_UP(ALIGN_DOWN(top_addr, PAGE_SIZE) / PAGE_SIZE / 8, PAGE_SIZE);

    kprintf("Total memory: %d bytes = %d kb = %d Mb = %d Gb.\n", info.memory_size, info.memory_size / 1024, info.memory_size / 1024 / 1024, info.memory_size / 1024 / 1024 / 1024);
    kprintf("Bitmap size: %d bytes.\n", bitmap_size);

    bitmap.size = bitmap_size;

    // search where to store the bitmap
    for(size_t i = 0; i < memmap->entry_count; i++){
        current_entry = memmap->entries[i];
        
        //skip unusable entries
        if(current_entry->type != LIMINE_MEMMAP_USABLE)
            continue;

        if(current_entry->length >= bitmap.size){
            kprintf("Found place for bitmap\n");

            bitmap.map = (uint8_t *)phys_addr_to_limine_virtual_addr(current_entry->base);

            //remove bitmap from available memory
            current_entry->base += bitmap_size;
            current_entry->length -= bitmap_size;
            break;
        }
    }

    // Set all memory to used
    memset((void *)bitmap.map, 0xff, bitmap.size);

    // Set usable zone as unused
    for(size_t i = 0; i < memmap->entry_count; i++){
        current_entry = memmap->entries[i];

        if(current_entry->type == LIMINE_MEMMAP_USABLE){
            free_pages((void *)current_entry->base, current_entry->length / PAGE_SIZE);
        }
    }

    kprintf("Pages: %d/%d\n", info.used_pages, info.max_pages);

    // null page is used
    bitmap_set_bit(&bitmap, 0);
}

void bitmap_set_bit(BITMAP_t *bitmap, int bit){
    bitmap->map[bit/8] |= (1 << (bit%8));
}

void bitmap_clear_bit(BITMAP_t *bitmap, int bit){
    bitmap->map[bit/8] &= ~(1 << (bit%8));
}

uint8_t bitmap_get_bit(BITMAP_t *bitmap, int bit){
    return bitmap->map[bit / 8] & (1 << (bit % 8));
}

void free_pages(void *pointer, size_t page_count){
    size_t index = limine_virtual_addr_to_phys_addr((uintptr_t)pointer) / PAGE_SIZE;

    for(size_t i = 0; i < page_count; i++){
        bitmap_clear_bit(&bitmap, index + i);
    }

    info.used_pages -= page_count;
}

void *find_free_pages(size_t requested_count){
    if(requested_count == 0){
        return NULL;
    }

    size_t index = 0;
    while(index < info.max_pages){
        int count = 0;
        while (count < requested_count && 
            !bitmap_get_bit(&bitmap, index + count)) 
        {
            count++;
        }

        // Found enough space
        if(count == requested_count){
            return (void *)BIT_TO_PAGE(index);
        }else{
            index += count + 1;
        }
    }

    // not enough memory
    return NULL;
}


void *alloc_page_phys_addr(size_t page_count){
    void *pointer = find_free_pages(page_count);

    if(pointer == NULL){
        return NULL;
    }

    size_t index = (size_t)pointer / PAGE_SIZE;

    for(size_t i = 0; i < page_count; i++){
        bitmap_set_bit(&bitmap, index + i);
    }

    info.used_pages += page_count;

    return (void *)(index * PAGE_SIZE);
}

void *alloc_page(size_t page_count){
    return (void*)phys_addr_to_limine_virtual_addr((uintptr_t)alloc_page_phys_addr(page_count));
}

void manually_alloc_page(void *ptr){
    uintptr_t addr = (uintptr_t) ptr;
    uintptr_t page = ALIGN_DOWN(addr, PAGE_SIZE);
    bitmap_set_bit(&bitmap, PAGE_TO_BIT(page));
}