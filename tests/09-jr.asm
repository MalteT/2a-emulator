#! mrasm

    CLR R0
LOOP:
    INC R0
    ST (0xFF), R0
    JR LOOP
