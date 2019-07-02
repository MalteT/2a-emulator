#! mrasm

.ORG 0

JR MAIN
JR INTERRUPT

MAIN:
    LDSP 0xEF
    EI
    CLR R0
LOOP:
    INC R0
    ST (0xFE), R0
    JR LOOP

INTERRUPT:
    ST (0xFF), R0
    RETI
