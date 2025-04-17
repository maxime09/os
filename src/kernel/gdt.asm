[bits 32]

section .text
global load_gdt

gdtr DW 0 ; limit storage
     DD 0 ; base storage

load_gdt:
    mov ax, [esp + 4]
    mov [gdtr], ax
    mov eax, [esp + 8]
    mov [gdtr+2], eax
    lgdt [gdtr]
    jmp 08h:flush

flush:
    mov ax, 10h
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    
    ret