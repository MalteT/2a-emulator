#! mrasm

    CLR R0
    CLR R1
LOOP:
    LD R0, (0xFC)
    LD R1, (0xFD)
    ADD R0, R1
    ST (0xFF), R0
    JR LOOP
