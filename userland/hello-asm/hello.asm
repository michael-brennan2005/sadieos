section .data
    ; Our message string followed by a newline character (0xA)
    msg db "Hello, world!", 0xA
    ; Calculate the length of the message
    len equ $ - msg

section .text
    ; Declare _start as the entry point for the linker
    global _start

_start:
    ; --- sys_write (syscall number 1) ---
    ; rax = 1 (syscall number for sys_write)
    mov rax, 1
    ; rdi = 1 (file descriptor for stdout)
    mov rdi, 1
    ; rsi = address of the message
    mov rsi, msg
    ; rdx = length of the message
    mov rdx, len
    ; Execute the syscall
    syscall

    ; --- sys_exit (syscall number 60) ---
    ; rax = 60 (syscall number for sys_exit)
    mov rax, 60
    ; rdi = 0 (exit status 0 for success)
    mov rdi, 0
    ; Execute the syscall
    syscall