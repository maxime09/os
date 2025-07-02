global usermode_switch
extern set_tss_rsp

usermode_switch:

  ; Set the segment registers to appropriate values for user mode
  mov rax, 0x43 ; user data segment
  mov ds, ax
  mov es, ax
  mov fs, ax
  mov gs, ax

  ;mov rax, 0x3B ; user code segment
  mov rax, rsp ; save stack pointer
  mov r9, rax
  mov r12, rdi

  mov rdi, 0
  mov rsi, rax
  mov rdx, 0
  call set_tss_rsp

  mov r8, 0x43
  push r8
  push r9
  mov rax, 0x202
  push rax
  mov rax, 0x3B 
  push rax
  push r12
  iretq