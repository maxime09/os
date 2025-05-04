#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "char_bmps.h"
#include "interrupts/interrupts.h"
#include <stdarg.h>
#include "interrupts/pic.h"
#include "mem/paging.h"

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

static inline void kputc(char c){
    if(c == '\n'){
        char_pos_x = 0;
        char_pos_y += 1;
    }else{
        kputc_at_pos(c, char_pos_x * CHAR_WIDTH, char_pos_y * CHAR_HEIGHT);
        char_pos_x++;
        if(char_pos_x >= char_per_row){
            char_pos_x = 0;
            char_pos_y++;
            if(char_pos_y >= rows_count){
                char_pos_y = 0;
            }
        }
    }
}

static inline void kputs(char *s){
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

void kmain(void){
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


    kputs("Hello, World!\n");
    struct limine_bootloader_info_response *bootloader_info = bootloader_info_request.response;

    kprintf("Bootloader: name: %s version: %s revision: %d\n", bootloader_info->name, bootloader_info->version, bootloader_info->revision);

    struct limine_hhdm_response *hhdm = hhdm_request.response;

    kprintf("HHDM offset: %x\n", hhdm->offset);
    hhdm_offset = hhdm->offset;

    memmap = memmap_request.response;


    pmm_init();
    vmm_init((uintptr_t)&kernel_ro_start, (uintptr_t)&kernel_ro_end, (uintptr_t)&kernel_wr_start, (uintptr_t)&kernel_wr_end);
    idt_init();

    for(size_t i = 0; i < 20; i++){
        void *ptr = alloc_page(1);
        kprintf("%d %x\n", i, ptr);
    }

    /*for(int i = 0; i < 32; i++){
        IRQ_clear_mask(i);
    }*/
    PIC_remap(0x20, 0x28);
    IRQ_clear_mask(1);

    // We're done, just hang...
    hcf();
}