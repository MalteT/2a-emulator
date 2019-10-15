#! mrasm

LOOP:
    CLR R0
    JZS OUTPUT
    MOV (0xFE), 0xFF
    JMP LOOP
OUTPUT:
    MOV (0xFF), 0b10101010
    JMP LOOP
