#! mrasm

; Used to verify which syntax version are supported by the .EQU asm instruction

    .EQU TEMP 42                ; Just a helper

    .EQU VAR TEMP               ; Allows using labels on both sides?
    .EQU VAR 42                 ; Allows using a number in decimal?
    .EQU VAR 0b00101010         ; Allows using a number in binary?
    .EQU VAR 0x2A               ; Allows using a number in hex?

    .EQU VAR 41                 ; Allows redefining labels?

    .EQU MAIN 42                ; Allows redefining labels that have been declared as jump markers?

MAIN:
    MOV (VAR), 27               ; Use like a label (1)?
    MOV (0xFE), (VAR)           ; Use like a label (2)?
    MOV (0xFF), VAR             ; Do crazy things, that would be useful but annoying with default labels?
