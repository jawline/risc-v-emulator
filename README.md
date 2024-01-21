# Risc-V Emulator

This project implements the base RV32I ISA. We do not include anything beyond the base user-mode ISA but it is sufficient to run programs compiled by GCC with some custom ecalls to observe IO.

## Hello World

The repository includes a hello world program in C that can be compiled with the gcc toolchain. A precompiled version is available under cli/test_programs/hello_world. A small assembly shim is used to initialize a small stack for it to execute in and ecall is used to putc whatever is in x10 to the cli. To executed it we run:
```
~/risc-v-emulator/cli master $ RUST_BACKTRACE=1 cargo run -- --program ./test_programs/hello_world
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/risc-v-emulator --program ./test_programs/hello_world`
Loading program
Writing program into memory from index 0
Executing
Hello World
```

## Tests

The instruction decoder is tested in `lib/src/instruction/decoder.rs`.

There are tests for each opcode under
`lib/src/cpu/instruction_sets/tests/rv32i.rs`'

The core fetch-decode-execute loop is tested under `lib/src/cpu/base.rs`. 
