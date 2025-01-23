     .ORIG x3000
     AND R0, R0, #0
LOOP ADD R0, R0, #1
     BRnp LOOP
     HALT
