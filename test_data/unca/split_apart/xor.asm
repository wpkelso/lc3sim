;; Set R3 to R1 ^ R2
;;    i.e.      OR(    AND(NOT(R1),R2),     AND(R1,NOT(R2)))
;;    i.e. NOT(AND(NOT(AND(NOT(R1),R2)),NOT(AND(R1,NOT(R2)))))
        .ORIG   x3000
xor     NOT     R1,R1
        AND     R3,R1,R2
        NOT     R1,R1
        NOT     R2,R2
        AND     R4,R1,R2
        NOT     R2,R2
        NOT     R3,R3
        NOT     R4,R4
        AND     R3,R3,R4
        NOT     R3,R3
        HALT
