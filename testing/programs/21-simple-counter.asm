#! mrasm

; This is the most basic example, just increment R0 indefinitely, writing the result to output FF!

    .ORG 0

LOOP:
    INC R0
    ST (0xFF), R0
    JR LOOP
