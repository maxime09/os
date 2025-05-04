#ifndef __PIC_H__
#define __PIC_H__

#include <stdint.h>

void IRQ_set_mask(uint8_t IRQline);
void IRQ_clear_mask(uint8_t IRQline);
void PIC_remap(int offset1, int offset2);
void pic_sendEOI(uint8_t irq);

#endif