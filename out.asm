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
    xor r8, r8          ; no minus sign
    jmp .convert_start

.negative:
    ; Try to negate – if overflow occurs, we have -2^63
    neg rax
    jo  .neg_min        ; overflow → rax was 0x8000000000000000
    mov r8, 1           ; normal negative, minus sign needed
    jmp .convert_start

.neg_min:
    ; rax is now 0x8000000000000000 (positive 2^63)
    ; keep it as is, set minus sign
    mov r8, 1
    jmp .convert_start

.positive:
    xor r8, r8

.convert_start:
    ; rax now holds the absolute value (unsigned)
    ; convert digits from least significant to most significant
.convert:
    xor rdx, rdx
    div rbx             ; rax = quotient, rdx = remainder
    add dl, '0'
    mov [rcx], dl
    dec rcx
    test rax, rax
    jnz .convert

    ; after loop, rcx points one byte before the first digit
    inc rcx             ; rcx points to the first digit

    ; insert minus sign if needed
    test r8, r8
    jz  .print
    dec rcx
    mov byte [rcx], '-'

.print:
    ; write to stdout (sys_write)
    mov rax, 1
    mov rdi, 1
    mov rsi, rcx
    mov rdx, buffer + 32
    sub rdx, rcx        ; length = (end of buffer) - start
    syscall

    pop rbx
    ret

;----------------------------------------------------------------------
; _start - example: compute 1 - 4 = -3 and print it
;----------------------------------------------------------------------
_start:
    mov rax, 1
    push rax
    mov rax, 4
    push rax
    pop rbx
    pop rax
    sub rax, rbx        ; rax = -3
    push rax
    add rsp, 8          ; discard the pushed value (stack cleanup)
    call print_int

    ; exit
    mov rax, 60
    xor rdi, rdi
    syscall
