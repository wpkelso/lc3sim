;; Set R0 to the number of bits "on" in R1
        .ORIG   x3000
pop     AND     R0,R0,#0
        ADD     R1,R1,#0       ;; test the msb
        BRzp    skipf
        ADD     R0,R0,#1
skipf   AND     R2,R2,#0
        ADD     R2,R2,#15
loop    ADD     R1,R1,R1       ;; now test the other 15
        BRzp    skip
        ADD     R0,R0,#1
skip    ADD     R2,R2,#-1
        BRp     loop
        HALT
        .END
