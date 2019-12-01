#! mrasm

    .ORG 0
    ;*STACKSIZE 16

    LDSP 0xEF
    CLR R0
LOOP:
    ST (0xFF), R0
    INC R0
    CALL LOOP
