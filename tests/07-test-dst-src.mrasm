#! mrasm

ADDR:
    .DB 42
INCEPTION:
    .DB ADDR
    ; .DB (ADDR)            ; does not work  (nur konstante erlaubt)
    ; .DB ((INCEPTION))     ; does not work  ( ((..)) nur mit reg)
    ; .DB (((01)))          ; does not work  ( ((..)) nur mit reg)

    LD R0, ADDR

LOOP:
    DEC R0
    INC R0
    DEC (R0)
    DEC (R0+)
    DEC ((R0+))
    DEC 0xFF
    DEC (0xFF)
DOOP:
    INC R0
    JZS LOOP
    JR LOOP
