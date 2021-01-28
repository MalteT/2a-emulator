#! mrasm
; Test the stacksize
; This will fail as soon as the program is overwritten by the stack
; This seems to happen when FF is at 229, but this is not a formal way to test
;
; $ 2a-emulator run ./05-stacksize-0.asm 10000 verify --ff 229

    .ORG 0
    *STACKSIZE 0

    LDSP 0xEF
    CLR R0
LOOP:
    ST (0xFF), R0
    INC R0
    CALL LOOP
