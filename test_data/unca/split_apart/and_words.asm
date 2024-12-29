;; AND's a vector of words
        .ORIG    x3000
andV    AND      R0,R0,#0
        ADD      R0,R0,#-1           ;; Set R0 to all one's
        LEA      R2,VECT             ;; Use LEA to get address of vector
        LD       R1,SIZE
        BRzp     STOPIT              ;; Return all one's if nothing to AND
LOOP    LDR      R3,R2,#0
        AND      R0,R0,R3
        ADD      R2,R2,#1
        ADD      R1,R1,#-1
        BRp      ALOOP
STOPIT  ST       R0,RESULT
        HALT
SIZE    .FILL    5
VECT    .FILL    xBEEF
        .FILL    x89AB
        .FILL    xFFFF
        .FILL    x89AB
        .FILL    x2008
RESULT  .BLKW    1       
        .END
