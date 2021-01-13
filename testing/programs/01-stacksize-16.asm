#! mrasm
; Test the default stacksize
; 
; $ 2a-emulator run ./01-stacksize-16.asm 10000 verify --ff 16

    .ORG 0
    ; This defaults to 16
    ; *STACKSIZE 16

    LDSP 0xEF
    CLR R0
LOOP:
    ST (0xFF), R0
    INC R0
    CALL LOOP
