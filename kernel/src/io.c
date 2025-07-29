#include "io.h"

void outb(uint16_t port, uint8_t value){
    __asm__ volatile ( "outb %b0, %w1" : : "a"(value), "Nd"(port) : "memory");
}

uint8_t inb(uint16_t port){
    uint8_t ret;
    __asm__ volatile ( "inb %w1, %b0" : "=a"(ret) : "Nd"(port) : "memory");
    return ret;
}

void outl(uint16_t port, uint32_t value){
    __asm__ volatile ( "outl %0, %w1" : : "a"(value), "Nd"(port) : "memory");
}

uint32_t inl(uint16_t port){
    uint32_t ret;
    __asm__ volatile ( "inl %w1, %0" : "=a"(ret) : "Nd"(port) : "memory");
    return ret;
}

void io_wait(void)
{
    outb(0x80, 0);
}
