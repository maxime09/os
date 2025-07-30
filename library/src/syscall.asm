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
    ret ; in case it fail (should not happens)

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

free:
    mov rsi, rdi
    mov rdi, 5
    int 0x40
    ret

putc:
    mov rsi, rdi
    mov rdi, 8
    int 0x40
    ret

move_cursor
    mov rdx, rsi
    mov rsi, rdi
    mov rdi, 7
    int 0x40
    ret