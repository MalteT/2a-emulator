#! mrasm

; Causes different board interrupts

    .ORG 0

    JR MAIN
    JR ISR


MAIN:
    ; Use input byte 0xFF to configure board interrupts
    ; but make sure the topmost bits are set.
    ; Do not do this if the input is zero
    LD R0, (0xFF)
    TST R0
    JZS SETUP_MICR
    BITS R0, 0b11000000
    ST (0xF2), R0

SETUP_MICR:
    ; Set interrupts all interrupts from the board will be passed through,
    ; additionally, key edge interrupts can be triggered aswell
    LDSP 0xEF
    BITS (0xF9), 0b00110001
    EI

SET_BOARD_OUTPUTS:
    ; Set all board outputs to max

    ; Set digital / analog outputs on the mr2da2
    MOV (0xF0), 0xFF
    MOV (0xF1), 0xFF

    ; Set ui/o directions to output
    MOV (0xF2), 0b10000111
    ; Set all ui/os to high
    MOV (0xF2), 0b00000111

LOOP:
    ; Loop forever
    JR LOOP


ISR: ; Interrupt subroutine
    MOV (0xFF), (0xF3) ; Output information about board interrupts
    MOV (0xFE), (0xF9) ; Output information about interrupts
    RETI
