section .bss
buffer resb 32
section .text
global _start
print_int:
push rbx
mov rcx, buffer + 31
mov byte [rcx], 10
dec rcx
mov rbx, 10
test rax, rax
jz  .zero_or_positive
js  .negative
jmp .positive
.zero_or_positive:
xor r8, r8
jmp .convert_start
.negative:
neg rax
jo  .neg_min
mov r8, 1
jmp .convert_start
.neg_min:
mov r8, 1
jmp .convert_start
.positive:
xor r8, r8
.convert_start:
.convert:
xor rdx, rdx
div rbx
add dl, '0'
mov [rcx], dl
dec rcx
test rax, rax
jnz .convert
inc rcx
test r8, r8
jz  .print
dec rcx
mov byte [rcx], '-'
.print:
mov rax, 1
mov rdi, 1
mov rsi, rcx
mov rdx, buffer + 32
sub rdx, rcx
syscall
pop rbx
ret
_start:
push rbp
mov rbp, rsp
sub rsp, 0
.start_while:
mov rax, 1
push rax
pop rax
cmp rax, 1
jne .after_body
; println
mov rax, 1
push rax
add rsp, 8
call print_int

jmp .start_while
.after_body:
mov rax, 60
xor rdi, rdi
syscall
