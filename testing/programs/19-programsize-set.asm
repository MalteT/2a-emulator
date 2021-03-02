#! mrasm

; Setting the programsize in this example should prevent the machine from halting normally. If it does, something is broken.

    *PROGRAMSIZE 5

    NOP         ; Address 0
    NOP         ; Address 1
    NOP         ; Address 2
    NOP         ; Address 3
    NOP         ; Address 4
    NOP         ; Address 5
    STOP        ; Address 6, in theory unreachable
