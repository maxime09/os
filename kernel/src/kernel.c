#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "char_bmps.h"
#include "interrupts/interrupts.h"
#include <stdarg.h>
#include "interrupts/pic.h"
#include "mem/paging.h"
#include "gdt.h"

#define LIMINE_API_REVISION 3

#include "limine.h"
#include "kernel.h"
#include "mem/pmm.h"

__attribute__((used, section(".limine_requests")))
static volatile LIMINE_BASE_REVISION(3);

__attribute__((used, section(".limine_requests")))
static volatile struct limine_framebuffer_request framebuffer_request = {
    .id = LIMINE_FRAMEBUFFER_REQUEST,
    .revision = 0
};

__attribute__((used, section(".limine_requests")))
static volatile struct limine_stack_size_request limine_stack_size_request = {
    .id = LIMINE_STACK_SIZE_REQUEST,
    .revision = 0,
    .stack_size = 64*1024,
};

__attribute__((used, section(".limine_requests")))
static volatile struct limine_memmap_request memmap_request = {
    .id = LIMINE_MEMMAP_REQUEST,
    .revision = 0
};

__attribute__((used, section(".limine_requests")))
static volatile struct limine_bootloader_info_request bootloader_info_request = {
    .id = LIMINE_BOOTLOADER_INFO_REQUEST,
    .revision = 0
};

__attribute__((used, section(".limine_requests")))
static volatile struct limine_hhdm_request hhdm_request = {
    .id = LIMINE_HHDM_REQUEST,
    .revision = 0
};

__attribute__((used, section(".limine_requests")))
static volatile struct limine_paging_mode_request paging_mode_request = {
    .id = LIMINE_PAGING_MODE_REQUEST,
    .revision = 0,
    .mode = LIMINE_PAGING_MODE_X86_64_4LVL,
    .max_mode = LIMINE_PAGING_MODE_X86_64_4LVL,
    .min_mode = LIMINE_PAGING_MODE_X86_64_4LVL
};

__attribute__((used, section(".limine_requests")))
static volatile struct limine_module_request module_request = {
    .id = LIMINE_MODULE_REQUEST,
    .revision = 0
};

__attribute__((used, section(".limine_requests")))
static volatile struct limine_mp_request mp_request = {
    .id = LIMINE_MP_REQUEST,
    .revision = 0,
    .flags = 0, // disable x2APIC
};


__attribute__((used, section(".limine_requests")))
static volatile struct limine_rsdp_request rsdp_request = {
    .id = LIMINE_RSDP_REQUEST,
    .revision = 3,
};


__attribute__((used, section(".limine_requests_start")))
static volatile LIMINE_REQUESTS_START_MARKER;

__attribute__((used, section(".limine_requests_end")))
static volatile LIMINE_REQUESTS_END_MARKER;

// Halt and catch fire function.
static void hcf(void) {
    for (;;) {
        asm ("hlt");
    }
}

struct limine_framebuffer *framebuffer;

uint64_t fb_width;
uint64_t fb_height;

static inline void kplot_pixel(uint32_t color, size_t x, size_t y){
    volatile uint32_t *fb_ptr = framebuffer->address;
    fb_ptr[y * (framebuffer->pitch / 4) + x] = color;
}

#define CHAR_WIDTH 8
#define CHAR_HEIGHT 8

static inline void kputc_at_pos(char c, size_t x, size_t y){
    static const uint32_t white = 0xffffff;
    static const uint32_t black = 0x000000;

    const uint8_t *bmp = font(c);

    for(size_t w = 0; w < CHAR_WIDTH; w++){
        for(size_t h = 0; h < CHAR_HEIGHT; h++){
            uint8_t mask = 1 << (w);
            if (bmp[h] & mask)
                kplot_pixel(white, x + w, y + h);
            else
                kplot_pixel(black, x + w, y + h);
        }
    }
}

size_t char_pos_x = 0;
size_t char_pos_y = 0;
size_t char_per_row;
size_t rows_count;

void move_cursor(size_t x, size_t y){
    char_pos_x = x;
    char_pos_y = y;
}

void kputc(char c){
    if(c == '\n'){
        char_pos_x = 0;
        char_pos_y += 1;
    }else{
        kputc_at_pos(c, char_pos_x * CHAR_WIDTH, char_pos_y * CHAR_HEIGHT);
        char_pos_x++;
        if(char_pos_x >= char_per_row){
            char_pos_x = 0;
            char_pos_y++;
        }
        if(char_pos_y >= rows_count){
            char_pos_y = 0;
        }
    }
}

void kputs(char *s){
    while(*s)
        kputc(*(s++));
}

void print_uint(uint64_t value){
    if(value == 0){
        kputc('0');
        return;
    }

    char buffer[21];
    size_t pos = 0;
    buffer[20] = 0;
    while(value != 0){
        buffer[19 - (pos++)] = ((char)(value % 10)) + '0';
        value /= 10;
    }
    kputs(&buffer[20 - pos]);
}

void print_hex(uint64_t value, size_t char_count){
    char *hex_chars = "0123456789abcdef";
    kputs("0x");
    if(value == 0){
        for(size_t i = 0; i < char_count; i++){
            kputc('0');
        }
        return;
    }
    char buffer[17];
    for(int i = 0; i < 16; i++){
        buffer[i] = '0';
    }
    size_t pos = 0;
    buffer[16] = 0;
    while(value != 0){
        buffer[15 - (pos++)] = hex_chars[value % 16];
        value /= 16;
    }
    kputs(&buffer[16 - char_count]);
}

void print_int(int64_t value){
    if(value < 0){
        kputc('-');
        print_uint((uint64_t)(-value));
    }else{
        print_uint((uint64_t)value);
    }
}

void kvprintf(const char *format, va_list list){
    for(size_t i = 0; format[i]; i++){
        if(format[i] == '%' && format[i+1]){
            switch (format[i+1]) {
                case 'd': 
                    print_int((int64_t) va_arg(list, int64_t));
                    break;
                case '%':
                    kputc('%');
                    break;
                case 'u':
                    print_uint((uint64_t) va_arg(list, uint64_t));
                    break;
                case 'c':
                    kputc((char) va_arg(list, int));
                    break;
                case 's':
                    kputs((char *) va_arg(list, char *));
                    break;
                case 'x':
                    print_hex((uint64_t) va_arg(list, uint64_t), 16);
            }
            i++;
        }else{
            kputc(format[i]);
        }
    }
}

void kprintf(const char *format, ...){
    va_list list;
    va_start(list, format);
    kvprintf(format, list);
    va_end(list);
}

struct limine_memmap_response *memmap;
size_t hhdm_offset;

extern uint8_t kernel_ro_start;
extern uint8_t kernel_ro_end;
extern uint8_t kernel_wr_start;
extern uint8_t kernel_wr_end;
extern uint8_t heap_start;
extern uint8_t heap_end;

extern void rust_kmain(void *, uintptr_t, void *);
extern void rust_slave_main(uint32_t, void *);
extern void init_alloc(uintptr_t, uintptr_t);

void kmain(void){

    __asm__ volatile("cli");

    // Ensure the bootloader actually understands our base revision (see spec).
    if (LIMINE_BASE_REVISION_SUPPORTED == false) {
        hcf();
    }

    // Ensure we got a framebuffer.
    if (framebuffer_request.response == NULL
     || framebuffer_request.response->framebuffer_count < 1) {
        hcf();
    }

    // Fetch the first framebuffer.
    framebuffer = framebuffer_request.response->framebuffers[0];
    fb_height = framebuffer->height;
    fb_width = framebuffer->width;
    char_per_row = fb_width / CHAR_WIDTH;
    rows_count = fb_height / CHAR_HEIGHT;

    gdt_init(0);
    kputs("GDT loaded\n");


    kprintf("CPU count: %d\n", mp_request.response->cpu_count);

    struct limine_bootloader_info_response *bootloader_info = bootloader_info_request.response;

    kprintf("Bootloader: name: %s version: %s revision: %d\n", bootloader_info->name, bootloader_info->version, bootloader_info->revision);
    kprintf("Rows count: %u\n", rows_count);
    kprintf("Paging mode: %u\n", paging_mode_request.response->mode);

    struct limine_module_response *module_response = module_request.response;
    kprintf("Module count: %d\n", module_response->module_count); 

    if(module_response->module_count == 0){
        kprintf("Missing initrd\n");
        hcf();
    }

    struct limine_file *initrd = module_response->modules[0];
    kprintf("%x %d\n", initrd->address, initrd->size);

    struct limine_hhdm_response *hhdm = hhdm_request.response;

    kprintf("HHDM offset: %x\n", hhdm->offset);
    hhdm_offset = hhdm->offset;

    memmap = memmap_request.response;

    uintptr_t initrd_start = (uintptr_t)initrd->address;
    uintptr_t initrd_end = initrd_start + initrd->size;

    pmm_init();
    kputs("Initialized physical memory manager\n");
    vmm_init((uintptr_t)&kernel_ro_start, (uintptr_t)&kernel_ro_end, (uintptr_t)&kernel_wr_start, (uintptr_t)&kernel_wr_end, initrd_start, initrd_end);
    kputs("Initialized virtual memory manager\n");
    idt_init();
    kputs("Initialized interrupt descriptor table\n");
    uintptr_t heap_start_addr = (uintptr_t)&heap_start;
    uintptr_t heap_end_addr = (uintptr_t)&heap_end;

    init_alloc(heap_start_addr, heap_end_addr);
    kputs("Initialized kernel memory allocator\n");

    PIC_remap(0x20, 0x28);
    kputs("Remapped PIC\n");
    IRQ_set_mask(0);
    IRQ_clear_mask(1);

    kputs("IRQ mask setup\n");

    __asm__ volatile("sti");
    kputs("Activated interrupts\n");

    

    rust_kmain(initrd->address, initrd->size, (void *)rsdp_request.response->address);

    // We're done, just hang...
    hcf();
}

void slave_core_kmain(struct limine_mp_info * mp_info){
    __asm__ volatile("cli");
    uint32_t core_id = mp_info->lapic_id;
    gdt_init(core_id);
    slave_core_init_vmm();
    slave_load_idt();
    rust_slave_main(core_id, (void *)rsdp_request.response->address);
    hcf();
}

void start_slave_core(void){
    struct limine_mp_response * response = mp_request.response;
    uint64_t cpu_count = response->cpu_count;
    limine_goto_address start_addr = slave_core_kmain;
    for(uint64_t i = 0; i < cpu_count && i < CPU_MAX_COUNT; i++){
        if(response->cpus[i]->lapic_id != response->bsp_lapic_id){
            response->cpus[i]->goto_address = start_addr;
        }
    }
}

extern void usermode_switch(uintptr_t addr, uintptr_t sp);

void jump_to_usermode(uintptr_t addr, uintptr_t sp){
    usermode_switch(addr, sp);
}
