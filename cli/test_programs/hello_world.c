void lexit() {
  asm volatile("addi x10,x0,0");
  asm volatile("ecall");
}

void lputc(char v) {
  asm volatile("addi x11,a0,0");
  asm volatile("addi x10,x0,1");
  asm volatile("ecall");
}

void lputs(char* s) {
  for (s; *s; s++) {
    lputc(*s);
  }
}

void c_start() {
  lputs("Hello World\n");
  lexit();
}
