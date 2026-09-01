#! mrasm
; Counts rising edges on UIO1 into the output register FF, by polling.
;
; No interrupts are involved: UIO1 is read straight out of the board status
; register at 0xF1. Used to check that a square wave driven onto UIO1 from
; outside actually reaches the machine.
;
; out  FF = number of rising edges seen so far, wraps at 256

    .ORG 0
    MOV     (0xF2), 0b10000000      ;UDR: UIO1..UIO3 are inputs
    CLR     R0                      ;event counter
    CLR     R2                      ;level seen on the previous pass
WAIT:
    LD      R1, (0xF1)              ;board status register
    BITC    R1, 0xFE                ;keep bit 0 only, which is UIO1
    CMP     R1, R2
    JZS     WAIT                    ;level unchanged
    MOV     R2, R1                  ;remember the new level
    TST     R1
    JZS     WAIT                    ;it fell to zero, so not a rising edge
    INC     R0
    ST      (0xFF), R0
    JR      WAIT
