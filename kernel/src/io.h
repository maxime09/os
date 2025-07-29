#ifndef __IO_H__
#define __IO_H__


#include "types.h"

void outb(uint16_t port, uint8_t value);
void outl(uint16_t port, uint32_t value);

uint8_t inb(uint16_t port);
uint32_t inl(uint16_t port);

void io_wait(void);

#endif