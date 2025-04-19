#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "limine.h"
#include "char_bmps.h"

__attribute__((used, section(".limine_requests")))
static volatile LIMINE_BASE_REVISION(3);

__attribute__((used, section(".limine_requests")))
static volatile struct limine_framebuffer_request framebuffer_request = {
    .id = LIMINE_FRAMEBUFFER_REQUEST,
    .revision = 0
};

__attribute__((used, section(".limine_requests")))
static volatile struct limine_memmap_request memmap_request = {
    .id = LIMINE_MEMMAP_REQUEST,
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

inline void kplot_pixel(uint32_t color, size_t x, size_t y){
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

static inline void kputc(char c){
    if(c == '\n'){
        char_pos_x = 0;
        char_pos_y += 1;
    }else{
        kputc_at_pos(c, char_pos_x * CHAR_WIDTH, char_pos_y * CHAR_HEIGHT);
        char_pos_x++;
        if(char_pos_x >= 80){
            char_pos_x = 0;
            char_pos_y++;
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

void print_int(int64_t value){
    if(value < 0){
        kputc('-');
        print_uint((uint64_t)(-value));
    }else{
        print_uint((uint64_t)value);
    }
}

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

    kputs("Hello, World!\n");
    struct limine_memmap_response *memmap = memmap_request.response;

    for(size_t i = 0; i < memmap->entry_count; i++){
        
    }
    print_int(-123456789101112);

    // We're done, just hang...
    hcf();
}