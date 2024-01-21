void lputs(char* s);

void c_start() {
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

