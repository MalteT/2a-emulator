#! mrasm
; Test the stacksize
; 
; $ 2a-emulator run ./03-stacksize-48.asm 10000 verify --ff 48

    .ORG 0
    *STACKSIZE 48

    LDSP 0xEF
    CLR R0
LOOP:
    ST (0xFF), R0
    INC R0
    CALL LOOP
