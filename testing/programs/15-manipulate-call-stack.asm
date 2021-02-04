#! mrasm

    .ORG 0
    JR INIT
    JR ISR


INIT:
    LDSP 0xEF
    BITS (0xF9), 1
    EI
MAIN:
    JR MAIN


_REACHABLE:
    BITS (0xFF), 1
    MOV (0xEC), UNREACHABLE
    RET


UNREACHABLE:
    ; WHAAAT
    BITS (0xFF), 2
    STOP


ISR:
    CALL _REACHABLE
