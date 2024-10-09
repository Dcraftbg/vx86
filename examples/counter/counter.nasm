[BITS 16]
mov ax, 4
condition:
cmp ax, 10000
jge loop_end
body:
add ax, 1
jmp condition
loop_end:

