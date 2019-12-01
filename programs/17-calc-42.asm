#! mrasm

JR MAIN             ; Start at MAIN
JR INTERRUPT        ; Interrupt routine

MAIN:
    EI              ; Enable interrupts
    LDSP 0xEF       ; Load stack pointer, to enable calls
    BITS (0xF9), 1  ; Enable key edge interrupts
    CLR R0          ; Clear register 0
LOOP:               ; The 'Adding'-loop
    INC R0          ; Increase R0
    CMP R0, 42      ; Compare with the target value
    JZS OUTPUT      ; Jump to OUTPUT on R0 == 42
    JR LOOP         ; If R0 != 42, keep adding

OUTPUT:
    ST (0xFF), R0   ; Move R0 to output register
    CLR R0          ; Clear R0
    JR LOOP         ; Return to loop

INTERRUPT:          ; Interrupt handling
    MOV (0xFF), 41  ; Store 41 in output register
    STOP            ; Stop the machine
    RETI            ; If continued, return from interrupt

