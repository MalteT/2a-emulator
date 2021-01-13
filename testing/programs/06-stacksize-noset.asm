#! mrasm
; Test the stacksize
; Since NOSET should leave the stacksize untouched, the default of 16 will be active
;
; $ 2a-emulator run ./06-stacksize-noset.asm 10000 verify --ff 16

    .ORG 0
    *STACKSIZE NOSET

    LDSP 0xEF
    CLR R0
LOOP:
    ST (0xFF), R0
    INC R0
    CALL LOOP
