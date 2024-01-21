use riscv_lib::cpu::base::Cpu;
use riscv_lib::memory::Memory;

fn main() {
    println!("Starting up\n");
    let mut mem = Memory::new(1024 * 1024);
    let mut cpu = Cpu::new();

    loop {
        cpu.step(&mut mem);
    }
}
