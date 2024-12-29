;; Counts the number of times a character occurs in a string
;; Character -- stored at x4000
;; String    -- stored at x5000
;; Result    -- stored at x6000
        .ORIG    x3000
nmChr   AND      R0,R0,#0
        LD       R1,AFILE           ;; R1 has address of the string
        LDI      R2,ALOOK4          ;; R2 has the value of the string
        NOT      R2,R2
        ADD      R2,R2,#1
ALOOP   LDR      R3,R1,#0
        BRz      STOPIT             ;; Leave loop on zero word
        ADD      R3,R3,R2
        BRnp     NOCOUNT
        ADD      R0,R0,#1
NOCOUNT ADD      R1,R1,#1
        BR       ALOOP
STOPIT  STI      R0,ACOUNT          ;; Count is stored
        HALT
ALOOK4  .FILL    x4000
AFILE   .FILL    x5000
ACOUNT  .FILL    x6000
        .END
