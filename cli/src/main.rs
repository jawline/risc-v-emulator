use clap::Parser;
use riscv_lib::cpu::base::Cpu;
use riscv_lib::memory::Memory;
use std::fs;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    program: String,

    #[arg(short, long, default_value_t = 1 << 17)]
    memory_bytes: usize,
}

fn read_file_as_bytes(path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let byte_content = fs::read(path)?;
    Ok(byte_content)
}

fn main() {
    let args = Args::parse();

    println!("Loading program");

    let program = read_file_as_bytes(&args.program).unwrap();

    let mut mem = Memory::new(args.memory_bytes);

    println!("Writing program into memory from index 0");

    for (index, &byte) in program.iter().enumerate() {
        mem.set8(index, byte).unwrap();
    }
    
    println!("Executing");

    let mut cpu = Cpu::new();

    // It seems like it is etiquette to
    // boot at address 0x200
    //cpu.state.registers.pc = 0x200;

    loop {
        cpu.step(&mut mem);
    }
}
