#! mrasm

; If this program is executed correctly, output register FF
; will contain 1 iff an interrupt was triggered

    .ORG 0

    JR MAIN
    JR ISR

MAIN:
    LDSP 0xEF
    BITS (0xF9), 1
    EI
LOOP:
    JR LOOP

ISR:
    ST (0xFF), 1
