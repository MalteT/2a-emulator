#! mrasm
; Test the stacksize
; 
; $ 2a-emulator run ./02-stacksize-32.asm 10000 verify --ff 32

    .ORG 0
    *STACKSIZE 32

    LDSP 0xEF
    CLR R0
LOOP:
    ST (0xFF), R0
    INC R0
    CALL LOOP
