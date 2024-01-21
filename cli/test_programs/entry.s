.global _start
_start:
  addi sp, x0, 2047
  call c_start
loop:
  j loop
