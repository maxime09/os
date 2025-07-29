#ifndef __INTERRUPTS_H__
#define __INTERRUPTS_H__

#include "stdint.h"


__attribute__((packed))
typedef struct interrupt_stack{
    uint64_t r15;
    uint64_t r14;
    uint64_t r13;
    uint64_t r12;
    uint64_t r11;
    uint64_t r10;
    uint64_t r9;
    uint64_t r8;
    uint64_t rsp;
    uint64_t rdi;
    uint64_t rbp;
    uint64_t rdx;
    uint64_t rcx;
    uint64_t rax;
    uint64_t rsi;
    uint64_t interrupt_code;
    uint64_t error_code;
    uint64_t instruction_pointer;
} interrupt_stack;


void interrupt_handler(interrupt_stack stack);
void idt_init();
void slave_load_idt();

#endif