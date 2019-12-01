#! mrasm

FIRST_LINE:
LOOP:
    LD R0, 42
    INC R0
    JZS LOOP
    JR FIRST_LINE

; Did work
