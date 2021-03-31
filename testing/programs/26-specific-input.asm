#! mrasm

; This program will error halt the machine if the following input is not present:
; FC = 1
; FD = 2
; FE = 3
; FF = 4

START:
    CMP (0xFC), 1
    JZC ERR
    CMP (0xFD), 2
    JZC ERR
    CMP (0xFE), 3
    JZC ERR
    CMP (0xFF), 4
    JZC ERR
    JR START

ERR:
    .DB 0

