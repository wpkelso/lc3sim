;; Set R0 to 10*R1
        .ORIG   x3000
mul10   ADD     R0,R1,R1      ; R0 ==  2*R1
        ADD     R0,R0,R0      ; R0 ==  4*R1
        ADD     R0,R0,R1      ; R0 ==  5*R1
        ADD     R0,R0,R0      ; R0 == 10*R1
        HALT
