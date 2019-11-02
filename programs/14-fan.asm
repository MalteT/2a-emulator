#! mrasm
    .ORG 0

    CLR R0
LOOP:
    INC R0
    ST (0xF0), R0
    JR LOOP
