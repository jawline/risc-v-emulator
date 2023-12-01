mod cpu_r32i;
mod instruction;
mod memory;
mod registers;

use cpu_r32i::Cpu;
use memory::Memory;

fn main() {
    println!("Starting up\n");
    let mut mem = Memory::new(1024 * 1024);
    let mut cpu = Cpu::new();
}
