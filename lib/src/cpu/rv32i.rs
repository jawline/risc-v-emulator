/**
 * This implements an RV32I RISC-V CPU core with an ecall that branches on x10
 * ECall behaviour:
 * x10 = 0: exit
 * x10 = 1: print x11 to stdout
 */
use crate::cpu::instruction_sets::rv32i::CpuState;
use crate::memory::Memory;
use crate::cpu::instruction_sets::rv32i::InstructionSet;

pub struct Cpu {
    pub state: CpuState,
    tbl: InstructionSet,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            state: CpuState::new(),
            tbl: InstructionSet::new(),
        }
    }

    pub fn step(&mut self, memory: &mut Memory) {
        let next_instruction = memory
            .get32(self.state.registers.pc as usize)
            .expect("PC out of range");
        self.tbl
            .step(&mut self.state, memory, next_instruction, |op| {
                match op.state.registers.get(10) {
                    0 => panic!("program terminated"),
                    1 => print!("{}", op.state.registers.get(11) as u8 as char),
                    _ => panic!("illegal ecall"),
                }
            });
    }
}

#[cfg(test)]
mod basic_tests {
    use super::*;
    use crate::instruction::encoder;

    #[test]
    fn test_create() {
        let _cpu = Cpu::new();
    }

    #[test]
    fn test_step() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(8);
        memory.set32(0, encoder::no_op().encode()).unwrap();
        cpu.step(&mut memory);
        assert_eq!(cpu.state.registers.pc, 4);
    }
}
