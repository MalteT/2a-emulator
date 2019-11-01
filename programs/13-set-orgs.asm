#! mrasm
    .ORG 0

    CLR R0
    CLR R1
LOOP:
    INC R0
    ADD R1, R0
    ST (0xF0), R0
    ST (0xF1), R1
    JR LOOP
