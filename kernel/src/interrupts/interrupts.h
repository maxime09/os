#ifndef __INTERRUPTS_H__
#define __INTERRUPTS_H__

#include "stdint.h"

void interrupt_handler(uint64_t interrupt_code, uint64_t error_code);
void idt_init();
void slave_load_idt();


#endif