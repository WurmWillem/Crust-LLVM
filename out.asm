section .bss
buffer resb 32
section .text
global _start
print_int:
mov rcx, buffer + 31
mov byte [rcx], 10
dec rcx
mov rbx, 10
.convert:
xor rdx, rdx
div rbx
add dl, '0'
mov [rcx], dl
dec rcx
test rax, rax
jnz .convert
inc rcx
mov rax, 1
mov rdi, 1
mov rsi, rcx
mov rdx, buffer + 32
sub rdx, rcx
syscall
ret
_start:
mov rax, 3
push rax
mov rax, 30
push rax
pop rbx
pop rax
div rbx
push rax
add rsp, 8
call print_int
mov rax, 60
xor rdi, rdi
syscall
