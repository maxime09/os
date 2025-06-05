global gdt_load

gdt_load:
  lgdt [rdi]

  ; Set the segment registers to appropriate values for kernel mode
  mov ax, 0x30 ; Kernel data segment
  mov ds, ax
  mov es, ax
  mov fs, ax
  mov gs, ax
  mov ss, ax
  pop rdi

  ;load tss
  mov ax, 0x48
  ltr ax

  mov rax, 0x28 ; Kernel code segment
  push rax
  push rdi
  retfq