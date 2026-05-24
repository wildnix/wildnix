bits 64
global _start

section .text
_start:
    mov rax, 1          ; SYS_WRITE
    mov rdi, msg
    mov rsi, msg_len
    syscall

.loop:
    jmp .loop

section .rodata
msg:
    db "hello from userland", 10
msg_len equ $ - msg