bits 64
global _start

section .text
_start:
.loop:
    mov rax, 2        ; SYS_READ_KEY
    syscall

    test rax, rax
    jz .loop

    mov [buf], al

    mov rax, 1        ; SYS_WRITE
    mov rdi, buf
    mov rsi, 1
    syscall

    jmp .loop

section .bss
buf:
    resb 1