#include "../string/print.h"
#include "multiboot.h"
#include "gdt.h"

void kernel_main(multiboot_info_t* mbd, unsigned int multiboot_magic)
{
    initgdt();

    terminal_initialize();

    terminal_writestring("Hello, kernel World!\n");

    if(multiboot_magic != MULTIBOOT_BOOTLOADER_MAGIC){
        printf("Incorrect multiboot magic value");
        return;
    }

    /* Check bit 6 to see if we have a valid memory map */
    if(!(mbd->flags >> 6 & 0x1)) {
        printf("invalid memory map given by GRUB bootloader");
        return;
    }
    
    /* Loop through the memory map and display the values */
    int i;
    for(i = 0; i < mbd->mmap_length; 
        i += sizeof(multiboot_memory_map_t)) 
    {
        multiboot_memory_map_t* mmmt = 
            (multiboot_memory_map_t*) (mbd->mmap_addr + i);

        printf("Start Addr: %x | Length: %x | Size: %x | Type: %u\n",
            mmmt->addr_low, mmmt->len_low, mmmt->size, mmmt->type);

        if(mmmt->type == MULTIBOOT_MEMORY_AVAILABLE) {
            /* 
             * Do something with this memory block!
             * BE WARNED that some of memory shown as availiable is actually 
             * actively being used by the kernel! You'll need to take that
             * into account before writing to memory!
             */
        }
    }
}