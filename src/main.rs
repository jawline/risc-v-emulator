#![feature(const_mut_refs)]
#![feature(generic_arg_infer)]
mod cpu_r32i;
mod instruction;
mod memory;
mod registers;

use crate::cpu_r32i::Cpu;
use crate::memory::Memory;

fn main() {
    println!("Starting up\n");
    let mut mem = Memory::new(1024 * 1024);
    let mut cpu = Cpu::new();
}
