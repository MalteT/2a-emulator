#! mrasm

; Sends the byte in 0xFF using the UART

    .ORG 0

    MOV (0xFA), (0xFF)
    STOP
