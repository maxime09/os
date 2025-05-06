#ifndef __PIC_H__
#define __PIC_H__

#include <stdint.h>

// those function are defined in rust-kernel/src/interrupts/pics.rs

void IRQ_set_mask(uint8_t IRQline);
void IRQ_clear_mask(uint8_t IRQline);
void PIC_remap(uint8_t offset1, uint8_t offset2);
void PIC_sendEOI(uint8_t irq);

#endif