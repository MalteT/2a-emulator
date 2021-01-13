#! mrasm
; Test the stacksize
; 
; $ 2a-emulator run ./04-stacksize-64.asm 10000 verify --ff 64

    .ORG 0
    *STACKSIZE 64

    LDSP 0xEF
    CLR R0
LOOP:
    ST (0xFF), R0
    INC R0
    CALL LOOP
