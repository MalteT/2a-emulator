#! mrasm

; Running this for 29 + 255 * 17 = 4364 cycles should result in FF = 0xFF
; So resetting the machine 743 cycles before the end of execution should result in FF = 42
; (i.e. running it for 1_000 cycles and resetting at cycle 257)

    JR MAIN
    JR ISR

MAIN:
    LDSP 0xEF
    BITS (0xF9), 1
    EI                  ; 29 cycles until here
LOOP:
    INC R0
    ST (0xFF), R0
    JR LOOP             ; 17 cycles for one iteration

ISR:
    ST (0xFE), R0
    RETI
