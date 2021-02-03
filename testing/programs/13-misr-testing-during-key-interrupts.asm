#! mrasm

    .ORG 0

    JR MAIN
    JR ISR

MAIN:
    LDSP 0xEF
    BITS (0xF9), 1
    EI

LOOP:
    ; Load MISR into R0
    LD R0, (0xF9)
    ; Now test R0 for the correct settings. If an interrupt disturbed the last command
    ; R0 will be reset in the ISR to prevent false errors
    BITT R0, 0b00010001
    JZC ERR_MISR_NOT_RESET
    JR LOOP

ISR:
    BITT (0xF9), 0b00010000
    JZS ERR_MISR_PENDING_NOT_SET
    BITT (0xF9), 0b00000001
    JZS ERR_MISR_REQUEST_NOT_SET
    ; To prevent false errors, reset R0, so that the main routine is not falsely
    ; assuming, that the MISR is not reset correctly. This is only relevant for
    ; the case, that an interrupt was triggered during LD R0, (0xF9)
    CLR R0
    RETI


ERR_MISR_NOT_RESET:
    MOV (0xFF), 1
    JR HALT
ERR_MISR_PENDING_NOT_SET:
    MOV (0xFF), 2
    JR HALT
ERR_MISR_REQUEST_NOT_SET:
    MOV (0xFF), 4
    JR HALT

HALT:
    STOP
