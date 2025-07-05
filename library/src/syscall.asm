global print
global exit
global input
global memalign

print:
    mov rsi, rdi
    mov rdi, 1
    int 0x40
    ret

exit:
    mov rsi, rdi
    mov rdi, 2
    int 0x40
    ; This syscall should not return

input:
    mov rdi, 3
    int 0x40
    ret

memalign:
    mov rdx, rsi
    mov rsi, rdi
    mov rdi, 4
    int 0x40
    ret