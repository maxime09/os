global common_interrupt_handler
global _isr_addr

%macro no_error_code_interrupt_handler 1
global interrupt_handler_%1
interrupt_handler_%1:
    push qword 0
    push rsi ; save old rsi
    xor rsi, rsi
    mov esi, [rsp+8]

    pusha64

    mov rdi, %1

    call interrupt_handler

    popa64

    pop rsi ; restore old rsi

    add rsp, 8

    iretq
%endmacro

%macro error_code_interrupt_handler 1
global interrupt_handler_%1
interrupt_handler_%1:
    push rsi ; save old rsi
    xor rsi, rsi
    mov esi, [rsp+8]

    pusha64

    mov rdi, %1

    call interrupt_handler

    popa64

    pop rsi ; restore old rsi

    add rsp, 8

    iretq
%endmacro

%macro ISR_ADDR 1
dq interrupt_handler_%1
%endmacro

extern interrupt_handler

%macro pusha64 0
    push rax
    push rbx
    push rcx
    push rdx
    push rbp
    push rdi
    push rsi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
%endmacro

%macro popa64 0
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rsi
    pop rdi
    pop rbp
    pop rdx
    pop rcx
    pop rbx
    pop rax
%endmacro



no_error_code_interrupt_handler 0
no_error_code_interrupt_handler 1
no_error_code_interrupt_handler 2
no_error_code_interrupt_handler 3
no_error_code_interrupt_handler 4
no_error_code_interrupt_handler 5
no_error_code_interrupt_handler 6
no_error_code_interrupt_handler 7
error_code_interrupt_handler    8
no_error_code_interrupt_handler 9
error_code_interrupt_handler    10
error_code_interrupt_handler    11
error_code_interrupt_handler    12
error_code_interrupt_handler    13
error_code_interrupt_handler    14
no_error_code_interrupt_handler 15
no_error_code_interrupt_handler 16
error_code_interrupt_handler    17
no_error_code_interrupt_handler 18
no_error_code_interrupt_handler 19
no_error_code_interrupt_handler 20
no_error_code_interrupt_handler 21
no_error_code_interrupt_handler 22
no_error_code_interrupt_handler 23
no_error_code_interrupt_handler 24
no_error_code_interrupt_handler 25
no_error_code_interrupt_handler 26
no_error_code_interrupt_handler 27
no_error_code_interrupt_handler 28
no_error_code_interrupt_handler 29
error_code_interrupt_handler    30
no_error_code_interrupt_handler 31
no_error_code_interrupt_handler 32
no_error_code_interrupt_handler 33


_isr_addr:
    ISR_ADDR 0
    ISR_ADDR 1
    ISR_ADDR 2
    ISR_ADDR 3
    ISR_ADDR 4
    ISR_ADDR 5
    ISR_ADDR 6
    ISR_ADDR 7
    ISR_ADDR 8
    ISR_ADDR 9
    ISR_ADDR 10
    ISR_ADDR 11
    ISR_ADDR 12
    ISR_ADDR 13
    ISR_ADDR 14
    ISR_ADDR 15
    ISR_ADDR 16
    ISR_ADDR 17
    ISR_ADDR 18
    ISR_ADDR 19
    ISR_ADDR 20
    ISR_ADDR 21
    ISR_ADDR 22
    ISR_ADDR 23
    ISR_ADDR 24
    ISR_ADDR 25
    ISR_ADDR 26
    ISR_ADDR 27
    ISR_ADDR 28
    ISR_ADDR 29
    ISR_ADDR 30
    ISR_ADDR 31
    ISR_ADDR 32
    ISR_ADDR 33