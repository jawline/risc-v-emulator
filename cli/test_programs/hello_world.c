void lputs(char* s);

void c_start() {
  asm volatile("mv a2, a0");
  lputs("Hello World\n");
}

void lputc(char v) {
  asm volatile("ecall");
}

void lputs(char* s) {
  for (s; *s; s++) {
    lputc(*s);
  }
}

