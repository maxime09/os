global print

print:
    mov rsi, rdi
    mov rdi, 1
    int 0x40
    ret