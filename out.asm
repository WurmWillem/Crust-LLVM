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
sub rsp, 8
; x declaration
mov rax, 0
push rax
pop rax
mov qword [rbp-8], rax

.start_while0:
; x < 100000000
mov rax, [rbp-8]
push rax
mov rax, 100000000
push rax
pop rbx
pop rax
cmp rax, rbx
setl al
movzx rax, al
push rax
pop rax
test rax, rax
jz .after_while_body0
; expression
; x + 1
mov rax, [rbp-8]
push rax
mov rax, 1
push rax
pop rbx
pop rax
add rax, rbx
push rax
pop rax
mov [rbp-8], rax
push rax
pop rax

jmp .start_while0
.after_while_body0:
; println
mov rax, [rbp-8]
push rax
call print_int

mov rax, 60
xor rdi, rdi
syscall
