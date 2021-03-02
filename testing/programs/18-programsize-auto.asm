#! mrasm

; This is the same as setting the programsize implicitly, which means the machine should error after leaving the program. To prevent the program from erroring because the `0`-instruction is executed, we'll have to do some magic ✨
; If this program halts without an error, something is wrong

    *PROGRAMSIZE AUTO

    .EQU TARGET_LOC 100   ; Put this at position 100
    .EQU TARGET_OPCODE 1  ; This is a STOP instruction

    LDSP 0xEF

    MOV (TARGET_LOC), TARGET_OPCODE
    CALL TARGET_LOC

