#! mrasm

; If the .EQU instruction works correctly, this program will set the output
; register FF and FE to 42.

.EQU FF         255
.EQU FE         254
.EQU NUMBER     33

    .ORG 0
    MOV (FF), (NUMBER)
    MOV (FE), (NUMBER)

    .ORG 33
    .DB 42
