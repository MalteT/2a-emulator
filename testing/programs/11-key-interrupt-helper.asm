#! mrasm

; This will test the MISR bits for the key interrupt.

    .ORG 0

    JR INIT
    JR ISR

INIT:
    LDSP 0xEF
    EI

MAIN:
    LD R0, (0xFC)
    JZS LOOP ; If input FC is zero, do not enable key interrupts
    BITS (0xF9), 0x01 ; Enable key interrupts

LOOP:
    MOV (0xFE), (0xF9)
    JR LOOP ; FOREVER!


ISR:
    MOV (0xFF), (0xF9)
    LD R0, (0xFD)
    JZS SKIP_STOP
    STOP
SKIP_STOP:
    RETI
